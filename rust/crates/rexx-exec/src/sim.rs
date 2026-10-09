/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! The deterministic simulation mode (spec 2026-10-07 section 4) and the
//! clock seam every clock read goes through. Each decision kind draws from a
//! stream of its own, so a knob turned on or off moves no other draw.
//! ```text
//! sim                    a seed from the clock, printed
//! sim:7                  seed 7, policy fifo, the seeded clock origin
//! sim:7,fifo,gc=0.01     a collection at each allocation with probability 0.01
//! sim:7,halt@500         every activity halted at clause boundary 500
//! sim:7,fail=wait:2      the second pinned wait fails with 11.1
//! sim:7,block=5          a command or native call on the baton is refused after 5 s
//! sim:7,clock=midnight   virtual time starts seconds before a local midnight
//! sim:7,clock=real       virtual time starts at the wall clock
//! ```

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime};

use crate::error::Failure;
use crate::scheduler::Posted;
use crate::{Interp, Loud};

/// The per-clause quantum's range: half and twice the oracle's measured
/// 58 ns per clause (`rexxcps`, median of five runs, 17,229,378 clauses per
/// second; `docs/superpowers/plans/phase-6-1-gate.md`, `## Task 8`).
const QUANTUM_NANOS: std::ops::RangeInclusive<u64> = 29..=116;

/// The seeded origin's epoch, 2026-01-01T00:00:00Z, in seconds since the Unix
/// epoch; a seed adds an offset under a year.
const EPOCH_SECONDS: u64 = 1_767_225_600;

const DAY_SECONDS: u64 = 86_400;

/// How a run in the simulation mode is driven: its seed, its policy and its
/// knobs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimConfig {
    pub seed: u64,
    pub policy: Policy,
    pub knobs: Knobs,
}

/// When the running activity is preempted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    /// Never: an activity runs until it parks or ends, and the ready queue
    /// is first in, first out.
    Fifo,
}

/// The simulation's knobs, each off by default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knobs {
    /// The probability of a collection at each allocation.
    pub gc: Option<f64>,
    /// The clause boundary, counting from 1, at which every activity is
    /// halted as a signal halts them.
    pub halt_at: Option<u64>,
    /// The pinned wait, counting from 1, that fails with 11.1 in place of
    /// waiting.
    pub fail_wait: Option<u64>,
    pub clock: ClockOrigin,
    /// The seconds of real time a command, or a native call made while
    /// another activity lives, may take on the baton, from 0 to
    /// [`BLOCK_LIMIT`].
    pub block: f64,
}

/// [`Knobs::block`] where `block=` does not set it.
const BLOCK_SECONDS: f64 = 2.0;

/// The largest `block=`: a day.
const BLOCK_LIMIT: f64 = 86_400.0;

/// Where virtual time starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockOrigin {
    /// A fixed epoch plus a seeded offset.
    Seeded,
    /// Seconds before a local day boundary, the day seeded.
    Midnight,
    /// The wall clock when the run starts.
    Real,
}

/// What a run in the simulation mode reports beside its outcome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimReport {
    pub seed: u64,
    pub policy: Policy,
    /// The clause boundaries counted.
    pub steps: u64,
    /// The switches from one activity to another.
    pub switches: u64,
    pub trace_hash: Option<u64>,
}

impl SimConfig {
    /// Reads `sim` or `sim:SEED[,item...]`; `sim` alone takes its seed from
    /// the clock.
    pub fn parse(text: &str) -> Result<SimConfig, String> {
        let mut config = SimConfig {
            seed: 0,
            policy: Policy::Fifo,
            knobs: Knobs {
                gc: None,
                halt_at: None,
                fail_wait: None,
                clock: ClockOrigin::Seeded,
                block: BLOCK_SECONDS,
            },
        };
        if text == "sim" {
            config.seed = clock_seed();
            return Ok(config);
        }
        let Some(rest) = text.strip_prefix("sim:") else {
            return Err(format!("`{text}` is not `sim` or `sim:SEED[,...]`"));
        };
        let mut items = rest.split(',');
        let seed = items.next().unwrap_or_default();
        config.seed = seed
            .parse()
            .map_err(|_| format!("`{seed}` is not a seed: a whole number below 2**64"))?;
        for item in items {
            let knobs = &mut config.knobs;
            if item == "fifo" {
                config.policy = Policy::Fifo;
            } else if let Some(q) = item.strip_prefix("gc=") {
                let q: f64 = q
                    .parse()
                    .ok()
                    .filter(|q| (0.0..=1.0).contains(q))
                    .ok_or_else(|| format!("`{item}`: gc is a probability from 0 to 1"))?;
                knobs.gc = Some(q);
            } else if let Some(k) = item.strip_prefix("halt@") {
                knobs.halt_at = Some(count(item, k)?);
            } else if let Some(k) = item.strip_prefix("fail=wait:") {
                knobs.fail_wait = Some(count(item, k)?);
            } else if let Some(seconds) = item.strip_prefix("block=") {
                knobs.block = seconds
                    .parse()
                    .ok()
                    .filter(|seconds: &f64| (0.0..=BLOCK_LIMIT).contains(seconds))
                    .ok_or_else(|| {
                        format!("`{item}`: block is a number of seconds from 0 to {BLOCK_LIMIT}")
                    })?;
            } else if item == "clock=midnight" {
                knobs.clock = ClockOrigin::Midnight;
            } else if item == "clock=real" {
                knobs.clock = ClockOrigin::Real;
            } else {
                return Err(format!(
                    "`{item}` is not a policy (`fifo`) or a knob (`gc=Q`, `halt@K`, \
                     `fail=wait:K`, `block=S`, `clock=midnight`, `clock=real`)"
                ));
            }
        }
        Ok(config)
    }

    /// The configuration a child `rexx` runs under: `seed`, this policy, and
    /// the knobs that are not scripted against this run's own clauses.
    fn child(self, seed: u64) -> SimConfig {
        SimConfig {
            seed,
            knobs: Knobs {
                halt_at: None,
                fail_wait: None,
                ..self.knobs
            },
            ..self
        }
    }
}

/// `K` of `halt@K` or `fail=wait:K`, counting from 1.
fn count(item: &str, k: &str) -> Result<u64, String> {
    k.parse()
        .ok()
        .filter(|k| *k > 0)
        .ok_or_else(|| format!("`{item}`: a count from 1"))
}

impl std::fmt::Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Fifo => f.write_str("fifo"),
        }
    }
}

impl std::fmt::Display for SimConfig {
    /// The text [`SimConfig::parse`] reads back.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sim:{},{}", self.seed, self.policy)?;
        let knobs = &self.knobs;
        if let Some(q) = knobs.gc {
            write!(f, ",gc={q}")?;
        }
        if let Some(k) = knobs.halt_at {
            write!(f, ",halt@{k}")?;
        }
        if let Some(k) = knobs.fail_wait {
            write!(f, ",fail=wait:{k}")?;
        }
        if knobs.block != BLOCK_SECONDS {
            write!(f, ",block={}", knobs.block)?;
        }
        match knobs.clock {
            ClockOrigin::Seeded => Ok(()),
            ClockOrigin::Midnight => f.write_str(",clock=midnight"),
            ClockOrigin::Real => f.write_str(",clock=real"),
        }
    }
}

/// A seed for `sim` alone.
fn clock_seed() -> u64 {
    #[expect(clippy::disallowed_methods, reason = "the seed of `sim` alone")]
    let since = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    since.as_secs() ^ u64::from(since.subsec_nanos()) << 32 ^ u64::from(std::process::id())
}

/// splitmix64's step: advances `state` and answers its next output.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// xoshiro256**.
pub(crate) struct Rng([u64; 4]);

impl Rng {
    /// A generator seeded with the next four outputs of the splitmix64 at
    /// `state`.
    fn split(state: &mut u64) -> Rng {
        Rng(std::array::from_fn(|_| splitmix64(state)))
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let s = &mut self.0;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// A draw from `[0, 1)`.
    pub(crate) fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A draw from `range`.
    pub(crate) fn within(&mut self, range: std::ops::RangeInclusive<u64>) -> u64 {
        let span = range.end() - range.start() + 1;
        range.start() + ((u128::from(self.next_u64()) * u128::from(span)) >> 64) as u64
    }
}

/// One stream per decision kind, each seeded from consecutive splitmix64
/// outputs of the seed, in this order.
#[expect(
    dead_code,
    reason = "`fifo` draws neither `schedule` nor `order`; both are split so the later streams keep their seeds"
)]
pub(crate) struct Streams {
    pub(crate) schedule: Rng,
    pub(crate) order: Rng,
    /// The clock's origin and quantum.
    pub(crate) clock: Rng,
    /// Collections at allocations.
    pub(crate) gc: Rng,
    /// The seeds of child `rexx` processes.
    pub(crate) children: Rng,
    /// The starting states of each activity's `RANDOM` generator.
    pub(crate) random: Rng,
}

impl Streams {
    fn new(seed: u64) -> Streams {
        let mut state = seed;
        Streams {
            schedule: Rng::split(&mut state),
            order: Rng::split(&mut state),
            clock: Rng::split(&mut state),
            gc: Rng::split(&mut state),
            children: Rng::split(&mut state),
            random: Rng::split(&mut state),
        }
    }
}

/// Virtual time: the instant and the wall time it started at, and how far
/// it has run.
pub(crate) struct Clock {
    origin: Instant,
    wall_origin: Duration,
    elapsed: Duration,
    quantum: Duration,
}

impl Clock {
    /// Draws the quantum and then the origin's offset from `stream`, both
    /// whatever `origin` is, so the knob moves no later draw.
    fn new(origin: ClockOrigin, stream: &mut Rng) -> Clock {
        let quantum = Duration::from_nanos(stream.within(QUANTUM_NANOS));
        let offset_micros = stream.within(0..=365 * DAY_SECONDS * 1_000_000 - 1);
        let before_midnight_micros = stream.within(500_000..=5_000_000);
        let epoch = Duration::from_secs(EPOCH_SECONDS);
        let wall_origin = match origin {
            ClockOrigin::Seeded => epoch + Duration::from_micros(offset_micros),
            ClockOrigin::Midnight => {
                let day = offset_micros / (DAY_SECONDS * 1_000_000);
                let midnight = epoch + Duration::from_secs((day + 1) * DAY_SECONDS);
                let utc = crate::builtin::datetime::UNIX_BASE_TIME + midnight.as_micros() as i64;
                let local = crate::builtin::datetime::local_offset_micros(utc);
                let at = midnight.as_micros() as i64 - local - before_midnight_micros as i64;
                Duration::from_micros(u64::try_from(at).unwrap_or_default())
            }
            ClockOrigin::Real =>
            {
                #[expect(clippy::disallowed_methods, reason = "`clock=real`'s origin")]
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
            }
        };
        #[expect(
            clippy::disallowed_methods,
            reason = "virtual instants count from here"
        )]
        let origin = Instant::now();
        Clock {
            origin,
            wall_origin,
            elapsed: Duration::ZERO,
            quantum,
        }
    }

    fn now(&self) -> Instant {
        self.origin + self.elapsed
    }

    fn wall_now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + self.wall_origin + self.elapsed
    }

    /// Moves virtual time on to `due`, or leaves it where it is past `due`.
    fn advance_to(&mut self, due: Instant) {
        if let Some(ahead) = due.checked_duration_since(self.now()) {
            self.elapsed += ahead;
        }
    }
}

/// A run's simulation state.
pub(crate) struct Sim {
    config: SimConfig,
    streams: Streams,
    clock: Clock,
    /// The pinned waits begun.
    pinned_waits: u64,
    /// Whether a wait nothing in the simulation can end was refused.
    stuck: bool,
    /// What another thread posted, where it posted something other than a
    /// signal's halt.
    breach: Option<&'static str>,
    /// Whether an inline native call outlasted `block=`.
    blocked_native: bool,
}

/// The watch on an inline native call in the simulation mode: a thread that
/// interrupts the interpreter's thread each `block=` the call outlasts.
pub(crate) struct Watch {
    stop: std::sync::mpsc::Sender<()>,
    watcher: std::thread::JoinHandle<bool>,
}

impl Interp {
    /// The clock every deadline is set and tested against: virtual time in
    /// the simulation mode, else the monotonic clock.
    #[inline]
    pub(crate) fn now(&self) -> Instant {
        match &self.sim {
            Some(sim) => sim.clock.now(),
            #[expect(clippy::disallowed_methods, reason = "the seam")]
            None => Instant::now(),
        }
    }

    /// The wall clock `TIME` and `DATE` read: virtual time in the simulation
    /// mode.
    #[inline]
    pub(crate) fn wall_now(&self) -> SystemTime {
        match &self.sim {
            Some(sim) => sim.clock.wall_now(),
            #[expect(clippy::disallowed_methods, reason = "the seam")]
            None => SystemTime::now(),
        }
    }

    /// Enters the simulation mode for the rest of the run.
    pub(crate) fn start_sim(&mut self, config: SimConfig) {
        let mut streams = Streams::new(config.seed);
        let clock = Clock::new(config.knobs.clock, &mut streams.clock);
        self.sim = Some(Box::new(Sim {
            config,
            streams,
            clock,
            pinned_waits: 0,
            stuck: false,
            breach: None,
            blocked_native: false,
        }));
        self.pool.set_bound(0);
        self.timer.requests().set(crate::timer::SIM);
        self.activity.random_source = None;
        if config.knobs.gc.is_some() {
            self.collect_due = true;
        }
    }

    /// A clause boundary in the simulation mode, the `clauses`-th: virtual
    /// time moves on a quantum, and `halt@K` halts at its boundary.
    #[cold]
    #[inline(never)]
    pub(crate) fn sim_clause(&mut self, clauses: u64) -> Result<(), Failure> {
        let Some(sim) = self.sim.as_deref_mut() else {
            return Ok(());
        };
        sim.clock.elapsed += sim.clock.quantum;
        let halt = sim.config.knobs.halt_at == Some(clauses);
        self.sim_breached()?;
        if halt {
            self.halt_all();
        }
        Ok(())
    }

    /// Fails with the refusal for a post or a callback another thread made,
    /// if one did.
    pub(crate) fn sim_breached(&mut self) -> Result<(), Failure> {
        let requests = self.timer.requests();
        if requests.pending(crate::timer::FOREIGN) {
            requests.clear(crate::timer::FOREIGN);
            return Err(Loud::sim_foreign_post("a callback").into());
        }
        if let Some(sim) = self.sim.as_deref_mut()
            && std::mem::take(&mut sim.blocked_native)
        {
            return Err(Loud::sim_blocked_native().into());
        }
        match self.sim.as_deref_mut().and_then(|sim| sim.breach.take()) {
            Some(what) => Err(Loud::sim_foreign_post(what).into()),
            None => Ok(()),
        }
    }

    /// Idles until `due` in the simulation mode: virtual time jumps there,
    /// unless the run's deadline, which is real, has passed.
    pub(crate) fn sim_idle_until(&mut self, due: Instant) -> Result<(), Failure> {
        if self.deadline_passed() {
            return Err(self.expire_deadline());
        }
        if let Some(sim) = self.sim.as_deref_mut() {
            sim.clock.advance_to(due);
        }
        self.drain_completions();
        self.sim_breached()
    }

    /// The refusal for a wait that only a signal can end.
    pub(crate) fn sim_endless_wait(&mut self) -> Failure {
        self.sim_stuck(Loud::sim_endless_wait())
    }

    /// The refusal for a wait for another thread's post, which a pool of
    /// bound 0 leaves none to make: every completion is posted by this
    /// thread before it is waited for.
    pub(crate) fn sim_unpostable_wait(&mut self) -> Failure {
        self.sim_stuck(Loud::scheduler_inconsistency(
            "a wait for another thread's post in the simulation mode",
        ))
    }

    fn sim_stuck(&mut self, loud: Loud) -> Failure {
        if let Some(sim) = self.sim.as_deref_mut() {
            sim.stuck = true;
        }
        loud.into()
    }

    /// Whether a wait nothing in the simulation can end was refused, which
    /// ends the program's wait for its activities.
    pub(crate) fn sim_is_stuck(&self) -> bool {
        self.sim.as_ref().is_some_and(|sim| sim.stuck)
    }

    /// Whether the pinned wait now beginning is the one `fail=wait:K` fails.
    pub(crate) fn sim_fails_wait(&mut self) -> bool {
        let Some(sim) = self.sim.as_deref_mut() else {
            return false;
        };
        sim.pinned_waits += 1;
        sim.config.knobs.fail_wait == Some(sim.pinned_waits)
    }

    /// Whether `gc=q`'s draw declines the collection due at an allocation,
    /// where neither the slot nor the byte trigger makes it due.
    pub(crate) fn sim_declines_collection(&mut self) -> bool {
        let Some(sim) = self.sim.as_deref_mut() else {
            return false;
        };
        let Some(q) = sim.config.knobs.gc else {
            return false;
        };
        if sim.streams.gc.unit() < q {
            return false;
        }
        self.heap.bytes_since() < self.bytes_due
            && !(self.heap.will_grow() && self.heap.slot_capacity() >= self.collect_at)
    }

    /// Whether `gc=q` keeps a collection due at every allocation.
    pub(crate) fn sim_collects_at_random(&self) -> bool {
        self.sim
            .as_ref()
            .is_some_and(|sim| sim.config.knobs.gc.is_some())
    }

    /// Notes a post in `posted` that another thread made. Under a pool of
    /// bound 0 the one post this thread makes is an inline native call's
    /// completion, and the one another thread may make is a signal's halt.
    pub(crate) fn sim_screen_posts(&mut self, posted: &VecDeque<Posted>) {
        let Some(sim) = self.sim.as_deref_mut() else {
            return;
        };
        for post in posted {
            let what = match post {
                Posted::Completed(_) | Posted::Halt => continue,
                Posted::Recall(_) => "a recall of the baton",
                Posted::Output { .. } => "a command's output",
                Posted::Unblocked { .. } => "a command's end",
                Posted::Input(_) => "a standard input chunk",
                Posted::Panicked(_) => "a pool thread's panic",
            };
            sim.breach.get_or_insert(what);
        }
    }

    /// The starting state of an activity's `RANDOM` generator in the
    /// simulation mode.
    pub(crate) fn sim_random_seed(&mut self) -> Option<u64> {
        let sim = self.sim.as_deref_mut()?;
        Some(sim.streams.random.next_u64())
    }

    /// The `REXX_SWITCH_MODE` a child process gets in the simulation mode,
    /// its seed drawn from the children stream.
    pub(crate) fn child_switch_mode(&mut self) -> Option<String> {
        let sim = self.sim.as_deref_mut()?;
        let seed = sim.streams.children.next_u64();
        Some(sim.config.child(seed).to_string())
    }

    /// Starts the watch on an inline native call, in the simulation mode.
    pub(crate) fn sim_watch_native(&self) -> Option<Watch> {
        let bound = self.sim_block_bound()?;
        let target = crate::signal::Interrupter::here();
        let (stop, stopped) = std::sync::mpsc::channel();
        let watcher = std::thread::spawn(move || {
            crate::signal::block();
            let mut fired = false;
            while let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = stopped.recv_timeout(bound)
            {
                target.interrupt();
                fired = true;
            }
            fired
        });
        Some(Watch { stop, watcher })
    }

    /// Ends `watch` once its call has returned; a call that outlasted
    /// `block=` is refused at the next boundary.
    pub(crate) fn sim_end_watch(&mut self, watch: Watch) {
        let _ = watch.stop.send(());
        let fired = watch.watcher.join().unwrap_or(true);
        if let Some(sim) = self.sim.as_deref_mut() {
            sim.blocked_native |= fired;
        }
    }

    /// How long a command's wait on the baton may take, in the simulation
    /// mode.
    pub(crate) fn sim_block_bound(&self) -> Option<Duration> {
        let sim = self.sim.as_deref()?;
        Some(Duration::from_secs_f64(sim.config.knobs.block))
    }

    /// The report a run in the simulation mode ends with.
    pub(crate) fn sim_report(&self) -> Option<SimReport> {
        let sim = self.sim.as_deref()?;
        Some(SimReport {
            seed: sim.config.seed,
            policy: sim.config.policy,
            steps: self.switch_clauses(),
            switches: self.activities.switches(),
            trace_hash: None,
        })
    }
}

impl Loud {
    /// A post or a callback in the simulation mode from a thread other than
    /// this one, other than a signal's halt, whose timing is real.
    pub(crate) fn sim_foreign_post(what: &str) -> Loud {
        Loud {
            message: crate::owned_message(
                &format!("{what} from another thread in the simulation mode"),
                None,
            ),
        }
    }

    /// A command whose wait on the baton outlasted `block=`'s real time,
    /// whether only another activity could end it or it is only slow.
    pub(crate) fn sim_blocked_command() -> Loud {
        Loud {
            message: crate::owned_message(
                "a command in the simulation mode that waits longer than its bound",
                None,
            ),
        }
    }

    /// A native call on the baton in the simulation mode that outlasted
    /// `block=`'s real time, while another activity lived.
    pub(crate) fn sim_blocked_native() -> Loud {
        Loud {
            message: crate::owned_message(
                "a native call in the simulation mode that runs longer than its bound",
                None,
            ),
        }
    }

    /// A wait in the simulation mode that nothing but a real signal can end.
    pub(crate) fn sim_endless_wait() -> Loud {
        Loud {
            message: crate::owned_message(
                "a wait in the simulation mode that only a signal can end",
                None,
            ),
        }
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "these tests time real runs")]
mod tests;
