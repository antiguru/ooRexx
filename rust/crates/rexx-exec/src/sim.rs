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
//! sim:7,pre:2,k=40       two preemptions among the first 40 contended steps
//! sim:7,uniform:0.2      a preemption at each contended step with probability 0.2
//! sim:7,pct:3,k=40       PCT of depth 3 over the first 40 contended steps
//! sim:7,order=fifo       the activities one event readies run in the order readied
//! sim:7,floor=1000       a preemption forced after 1000 contended steps
//! sim:7,trace=FILE       the decision trace written to FILE
//! sim:replay=FILE        the decisions FILE holds taken in place of drawn
//! ```

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
/// knobs, and where its decision trace is written or read from.
#[derive(Clone, Debug, PartialEq)]
pub struct SimConfig {
    pub seed: u64,
    pub policy: Policy,
    pub order: Order,
    pub knobs: Knobs,
    /// The file the run's decision trace is written to.
    pub trace: Option<PathBuf>,
    /// The trace the run takes its decisions from in place of drawing them.
    pub replay: Option<Replay>,
}

/// When the running activity is preempted at a contended step, a clause
/// boundary where another activity is ready. Every policy also preempts
/// where the fairness floor forces it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Policy {
    /// Only where the floor forces it.
    Fifo,
    /// At `d` contended steps drawn uniformly among the first `k`.
    Pre { d: u32, k: u64 },
    /// At each contended step with probability `p`.
    Uniform { p: f64 },
    /// PCT (spec ruling R5): seeded priorities at spawn, `d - 1` change
    /// points drawn among the first `k` contended steps that lower the
    /// running activity's priority, and the ready activity of the highest
    /// priority runs. The floor lowers the activity it preempts below every
    /// other.
    Pct { d: u32, k: u64 },
}

/// The order of the activities one event readied: a post, a release, a
/// halt or a timer's end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// The order they were readied in.
    Fifo,
    /// A seeded shuffle.
    OneEvent,
}

/// One decision a run took, in the order it took them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// A preemption at this contended step, counting from 1.
    Preempt(u64),
    /// An index into the ready queue: a step of a one-event shuffle, or the
    /// activity a priority pick took.
    Pick(u64),
    /// A collection at this allocation `gc=` drew for, counting from 1.
    Collect(u64),
}

/// A trace read back for a replay.
#[derive(Clone, Debug, PartialEq)]
pub struct Replay {
    pub path: PathBuf,
    pub decisions: Arc<[Decision]>,
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
    /// The seconds of real time a command or a native call may take on the
    /// baton, from [`BLOCK_FLOOR`] to [`BLOCK_LIMIT`].
    pub block: f64,
    /// The contended steps an activity runs before a preemption is forced.
    pub floor: u64,
}

/// The smallest `block=`: a millisecond.
const BLOCK_FLOOR: f64 = 0.001;

/// [`Knobs::block`] where `block=` does not set it.
const BLOCK_SECONDS: f64 = 2.0;

/// The largest `block=`: a day.
const BLOCK_LIMIT: f64 = 86_400.0;

/// [`Knobs::floor`] where `floor=` does not set it: the oracle's 24 ms slice
/// (`SLICE_LENGTH`) at its measured 58 ns per clause.
pub(crate) const FAIRNESS_FLOOR: u64 = 24_000_000 / 58;

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
    /// The contended steps among them.
    pub contended: u64,
    /// The switches from one activity to another.
    pub switches: u64,
    /// The hash of the decisions the run took, FNV-1a over each one's tag
    /// byte and its value's eight bytes, little endian.
    pub trace_hash: Option<u64>,
}

impl SimConfig {
    /// Reads `sim`, `sim:SEED[,item...]` or `sim:replay=FILE[,trace=FILE]`;
    /// `sim` alone takes its seed from the clock. A path holds no comma.
    pub fn parse(text: &str) -> Result<SimConfig, String> {
        let mut config = SimConfig {
            seed: 0,
            policy: Policy::Fifo,
            order: Order::OneEvent,
            knobs: Knobs {
                gc: None,
                halt_at: None,
                fail_wait: None,
                clock: ClockOrigin::Seeded,
                block: BLOCK_SECONDS,
                floor: FAIRNESS_FLOOR,
            },
            trace: None,
            replay: None,
        };
        if text == "sim" {
            config.seed = clock_seed();
            return Ok(config);
        }
        let Some(rest) = text.strip_prefix("sim:") else {
            return Err(format!("`{text}` is not `sim` or `sim:SEED[,...]`"));
        };
        let mut items = rest.split(',');
        let first = items.next().unwrap_or_default();
        if let Some(path) = first.strip_prefix("replay=") {
            let replay = nonempty_path(first, path)?;
            let mut trace = None;
            let mut previous = first;
            for item in items {
                if item.starts_with("replay=") {
                    return Err(format!("`{item}`: a second `replay=`"));
                }
                let Some(path) = item.strip_prefix("trace=") else {
                    return Err(comma_in_path(previous, item));
                };
                if trace.is_some() {
                    return Err(format!("`{item}`: a second `trace=`"));
                }
                trace = Some(nonempty_path(item, path)?);
                previous = item;
            }
            config = read_replay(&replay)?;
            config.trace = trace;
            return Ok(config);
        }
        config.seed = first
            .parse()
            .map_err(|_| format!("`{first}` is not a seed: a whole number below 2**64"))?;
        let mut items = items.peekable();
        let mut previous = first;
        while let Some(item) = items.next() {
            let knobs = &mut config.knobs;
            if item == "fifo" {
                config.policy = Policy::Fifo;
            } else if let Some(d) = item.strip_prefix("pre:") {
                let (d, k) = depth_and_steps(item, d, items.next())?;
                config.policy = Policy::Pre { d, k };
            } else if let Some(d) = item.strip_prefix("pct:") {
                let (d, k) = depth_and_steps(item, d, items.next())?;
                config.policy = Policy::Pct { d, k };
            } else if let Some(p) = item.strip_prefix("uniform:") {
                config.policy = Policy::Uniform {
                    p: probability(item, p)?,
                };
            } else if item == "order=fifo" {
                config.order = Order::Fifo;
            } else if let Some(q) = item.strip_prefix("gc=") {
                knobs.gc = Some(probability(item, q)?);
            } else if let Some(k) = item.strip_prefix("halt@") {
                knobs.halt_at = Some(count(item, k)?);
            } else if let Some(k) = item.strip_prefix("fail=wait:") {
                knobs.fail_wait = Some(count(item, k)?);
            } else if let Some(f) = item.strip_prefix("floor=") {
                knobs.floor = count(item, f)?;
            } else if let Some(seconds) = item.strip_prefix("block=") {
                knobs.block = seconds
                    .parse()
                    .ok()
                    .filter(|seconds: &f64| (BLOCK_FLOOR..=BLOCK_LIMIT).contains(seconds))
                    .ok_or_else(|| {
                        format!(
                            "`{item}`: block is a number of seconds from {BLOCK_FLOOR} to \
                             {BLOCK_LIMIT}"
                        )
                    })?;
            } else if item == "clock=midnight" {
                knobs.clock = ClockOrigin::Midnight;
            } else if item == "clock=real" {
                knobs.clock = ClockOrigin::Real;
            } else if let Some(path) = item.strip_prefix("trace=") {
                if config.trace.is_some() {
                    return Err(format!("`{item}`: a second `trace=`"));
                }
                config.trace = Some(nonempty_path(item, path)?);
            } else if previous.starts_with("trace=") {
                return Err(comma_in_path(previous, item));
            } else {
                return Err(format!(
                    "`{item}` is not a policy (`fifo`, `pre:D,k=N`, `uniform:P`, `pct:D,k=N`) \
                     or a knob (`order=fifo`, `gc=Q`, `halt@K`, `fail=wait:K`, `block=S`, \
                     `floor=F`, `clock=midnight`, `clock=real`, `trace=FILE`)"
                ));
            }
            previous = item;
        }
        Ok(config)
    }

    /// The configuration a child `rexx` runs under: `seed`, this policy, and
    /// the knobs that are not scripted against this run's own clauses or
    /// files.
    fn child(&self, seed: u64) -> SimConfig {
        SimConfig {
            seed,
            knobs: Knobs {
                halt_at: None,
                fail_wait: None,
                ..self.knobs
            },
            trace: None,
            replay: None,
            ..self.clone()
        }
    }

    /// This configuration without its trace file and replay: the first line
    /// of a trace file.
    fn header(&self) -> SimConfig {
        SimConfig {
            trace: None,
            replay: None,
            ..self.clone()
        }
    }
}

/// `D` and the `k=N` item after it, of `pre:D,k=N` or `pct:D,k=N`.
fn depth_and_steps(item: &str, d: &str, next: Option<&str>) -> Result<(u32, u64), String> {
    let d = d
        .parse()
        .ok()
        .filter(|d| *d > 0)
        .ok_or_else(|| format!("`{item}`: a depth from 1"))?;
    let Some(k) = next.and_then(|next| next.strip_prefix("k=")) else {
        return Err(format!("`{item}` needs `k=N` after it"));
    };
    Ok((d, count(item, k)?))
}

/// The path of `trace=FILE` or `replay=FILE`, refused where empty.
fn nonempty_path(item: &str, path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err(format!("`{item}`: an empty path"));
    }
    Ok(PathBuf::from(path))
}

/// The refusal of `item`, which follows the path item `path_item` and is no
/// item there: the path's tail, cut at a comma.
fn comma_in_path(path_item: &str, item: &str) -> String {
    format!("`{path_item},{item}`: a path may not contain a comma")
}

/// A probability from 0 to 1.
fn probability(item: &str, p: &str) -> Result<f64, String> {
    p.parse()
        .ok()
        .filter(|p| (0.0..=1.0).contains(p))
        .ok_or_else(|| format!("`{item}`: a probability from 0 to 1"))
}

/// `K` of `halt@K`, `fail=wait:K`, `floor=F` or `k=N`, counting from 1.
fn count(item: &str, k: &str) -> Result<u64, String> {
    k.parse()
        .ok()
        .filter(|k| *k > 0)
        .ok_or_else(|| format!("`{item}`: a count from 1"))
}

/// The configuration a trace file's first line holds, replaying the
/// decisions on its other lines. The first line is `CONFIG hash=H`, `H` the
/// decisions' [`trace_hash`] in 16 hex digits; a file whose decisions do not
/// hash to `H` is refused.
fn read_replay(path: &Path) -> Result<SimConfig, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("`{}`: {error}", path.display()))?;
    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default();
    let not_a_header = || {
        format!(
            "`{}`: `{header}` is not a trace's header, `CONFIG hash=H`",
            path.display()
        )
    };
    let (configuration, hash) = header.rsplit_once(" hash=").ok_or_else(not_a_header)?;
    let hash = (hash.len() == 16)
        .then(|| u64::from_str_radix(hash, 16).ok())
        .flatten()
        .ok_or_else(not_a_header)?;
    let mut config = SimConfig::parse(configuration)?;
    if config.trace.is_some() || config.replay.is_some() || configuration == "sim" {
        return Err(not_a_header());
    }
    let decisions = lines
        .map(|line| {
            let (kind, value) = line.split_once(' ').unwrap_or((line, ""));
            let value: u64 = value
                .parse()
                .map_err(|_| format!("`{}`: `{line}` is not a decision", path.display()))?;
            match kind {
                "preempt" => Ok(Decision::Preempt(value)),
                "pick" => Ok(Decision::Pick(value)),
                "collect" => Ok(Decision::Collect(value)),
                _ => Err(format!("`{}`: `{line}` is not a decision", path.display())),
            }
        })
        .collect::<Result<Arc<[Decision]>, String>>()?;
    let held = trace_hash(&decisions);
    if held != hash {
        return Err(format!(
            "`{}`: the decisions hash to {held:016x}, the header holds {hash:016x}",
            path.display()
        ));
    }
    config.replay = Some(Replay {
        path: path.to_path_buf(),
        decisions,
    });
    Ok(config)
}

impl std::fmt::Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Policy::Fifo => f.write_str("fifo"),
            Policy::Pre { d, k } => write!(f, "pre:{d},k={k}"),
            Policy::Uniform { p } => write!(f, "uniform:{p}"),
            Policy::Pct { d, k } => write!(f, "pct:{d},k={k}"),
        }
    }
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Preempt(step) => write!(f, "preempt {step}"),
            Decision::Pick(index) => write!(f, "pick {index}"),
            Decision::Collect(at) => write!(f, "collect {at}"),
        }
    }
}

impl std::fmt::Display for SimConfig {
    /// The text [`SimConfig::parse`] reads back.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(replay) = &self.replay {
            write!(f, "sim:replay={}", replay.path.display())?;
            if let Some(path) = &self.trace {
                write!(f, ",trace={}", path.display())?;
            }
            return Ok(());
        }
        write!(f, "sim:{},{}", self.seed, self.policy)?;
        if self.order == Order::Fifo {
            f.write_str(",order=fifo")?;
        }
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
        if knobs.floor != FAIRNESS_FLOOR {
            write!(f, ",floor={}", knobs.floor)?;
        }
        match knobs.clock {
            ClockOrigin::Seeded => {}
            ClockOrigin::Midnight => f.write_str(",clock=midnight")?,
            ClockOrigin::Real => f.write_str(",clock=real")?,
        }
        if let Some(path) = &self.trace {
            write!(f, ",trace={}", path.display())?;
        }
        Ok(())
    }
}

/// FNV-1a over each decision's tag byte and its value's eight bytes, little
/// endian: the same on every build and platform.
fn trace_hash(decisions: &[Decision]) -> u64 {
    let mut trace = Trace::new(false);
    for decision in decisions {
        trace.record(*decision);
    }
    trace.hash
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
pub(crate) struct Streams {
    /// Preemptions, `pre:`'s and `pct:`'s points, and `pct:`'s priorities.
    pub(crate) schedule: Rng,
    /// One-event shuffles.
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

/// The decisions a run took: their hash, and the decisions themselves where
/// a trace file is written.
struct Trace {
    hash: u64,
    kept: Option<Vec<Decision>>,
}

impl Trace {
    const OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;

    fn new(keep: bool) -> Trace {
        Trace {
            hash: Trace::OFFSET,
            kept: keep.then(Vec::new),
        }
    }

    fn record(&mut self, decision: Decision) {
        let (tag, value) = match decision {
            Decision::Preempt(step) => (1u8, step),
            Decision::Pick(index) => (2, index),
            Decision::Collect(at) => (3, at),
        };
        for byte in std::iter::once(tag).chain(value.to_le_bytes()) {
            self.hash = (self.hash ^ u64::from(byte)).wrapping_mul(Trace::PRIME);
        }
        if let Some(kept) = &mut self.kept {
            kept.push(decision);
        }
    }
}

/// A replay's place in the decisions it reads back, taken in the order the
/// file holds them. The first decision the run takes otherwise is refused,
/// and after it the run takes no decision from the file.
struct Replaying {
    path: PathBuf,
    decisions: Arc<[Decision]>,
    /// The next decision to take.
    next: usize,
    /// The positions in `decisions` of the preemptions and of the
    /// collections, each with the next one not yet taken.
    preempts: Vec<usize>,
    collects: Vec<usize>,
    next_preempt: usize,
    next_collect: usize,
    /// Whether the run took a decision the file does not hold.
    diverged: bool,
    /// That divergence's refusal, until a boundary raises it.
    refusal: Option<Loud>,
}

/// A decision keyed to a position the run counts: a contended step or an
/// allocation.
#[derive(Clone, Copy)]
enum Point {
    Preempt,
    Collect,
}

impl Replaying {
    fn new(replay: &Replay) -> Replaying {
        let at = |wanted: fn(&Decision) -> bool| {
            (replay.decisions.iter().enumerate())
                .filter(|(_, decision)| wanted(decision))
                .map(|(at, _)| at)
                .collect()
        };
        Replaying {
            path: replay.path.clone(),
            decisions: Arc::clone(&replay.decisions),
            next: 0,
            preempts: at(|decision| matches!(decision, Decision::Preempt(_))),
            collects: at(|decision| matches!(decision, Decision::Collect(_))),
            next_preempt: 0,
            next_collect: 0,
            diverged: false,
            refusal: None,
        }
    }

    /// Whether the trace takes `kind`'s decision at position `at`; positions
    /// come in increasing order, one at a time. `forced` is a preemption the
    /// floor takes whatever the trace holds.
    fn point(&mut self, kind: Point, at: u64, forced: bool) -> bool {
        if self.diverged {
            return forced;
        }
        let (positions, cursor, did, without) = match kind {
            Point::Preempt => (
                &self.preempts,
                &mut self.next_preempt,
                format!("preempts at contended step {at}"),
                format!("reaches contended step {at} without preempting at"),
            ),
            Point::Collect => (
                &self.collects,
                &mut self.next_collect,
                format!("collects at allocation {at}"),
                format!("reaches allocation {at} without collecting at"),
            ),
        };
        if let Some(&position) = positions.get(*cursor) {
            let point = match self.decisions[position] {
                Decision::Preempt(point) | Decision::Collect(point) | Decision::Pick(point) => {
                    point
                }
            };
            if point < at {
                self.diverge(position, format!("{without} {point}"));
                return forced;
            }
            if point == at {
                if position != self.next {
                    self.diverge(self.next, did);
                    return forced;
                }
                *cursor += 1;
                self.next += 1;
                return true;
            }
        }
        if forced {
            self.diverge(self.next, did);
        }
        forced
    }

    /// The next pick, from 0 to `last`.
    fn pick(&mut self, last: u64) -> u64 {
        if self.diverged {
            return 0;
        }
        match self.decisions.get(self.next) {
            Some(Decision::Pick(index)) if *index <= last => {
                self.next += 1;
                *index
            }
            _ => {
                self.diverge(self.next, format!("picks from 0 to {last}"));
                0
            }
        }
    }

    /// Refuses the decisions left where the run has ended.
    fn finish(&mut self) {
        if !self.diverged && self.next < self.decisions.len() {
            self.diverge(self.next, "ends".into());
        }
    }

    /// Notes that the run, at the decision at `at` in the file, did `did`.
    fn diverge(&mut self, at: usize, did: String) {
        let held = match self.decisions.get(at) {
            Some(decision) => format!("`{decision}`"),
            None => "no more".into(),
        };
        self.diverged = true;
        self.refusal = Some(Loud::sim_replay_diverged(format!(
            "the replay of `{}` diverged at decision {}: the file holds {held}, the run {did}",
            self.path.display(),
            at + 1
        )));
    }
}

/// Whether the sorted `points` hold `at`, moving `next` past every point up
/// to it.
fn reached(points: &[u64], next: &mut usize, at: u64) -> bool {
    while points.get(*next).is_some_and(|point| *point < at) {
        *next += 1;
    }
    if points.get(*next) == Some(&at) {
        *next += 1;
        return true;
    }
    false
}

/// `count` distinct contended steps drawn uniformly from `1..=k`, sorted.
fn draw_points(stream: &mut Rng, count: u64, k: u64) -> Vec<u64> {
    if count >= k {
        return (1..=k).collect();
    }
    let mut points = Vec::new();
    while (points.len() as u64) < count {
        let point = stream.within(1..=k);
        if !points.contains(&point) {
            points.push(point);
        }
    }
    points.sort_unstable();
    points
}

/// Sets `pct:`'s priority of the activity under `handle`.
fn set_priority(priorities: &mut Vec<i64>, handle: usize, priority: i64) {
    if priorities.len() <= handle {
        priorities.resize(handle + 1, PRIORITY_BASE);
    }
    priorities[handle] = priority;
}

/// Above every priority a change point sets, which counts down from `d - 1`.
const PRIORITY_BASE: i64 = 1 << 33;

/// The policy's state: the contended steps counted, and the points and
/// priorities `pre:` and `pct:` drew.
struct Schedule {
    contended: u64,
    /// The running activity's contended steps since it began running or
    /// the floor last preempted it.
    run_contended: u64,
    /// The switch count when `run_contended` last began.
    seen_switches: u64,
    /// `pre:`'s preemptions or `pct:`'s change points.
    points: Vec<u64>,
    next_point: usize,
    /// `pct:`'s change points passed.
    changes: i64,
    /// `pct:`'s priorities, by activity handle.
    priorities: Vec<i64>,
}

/// A run's simulation state.
pub(crate) struct Sim {
    config: SimConfig,
    streams: Streams,
    clock: Clock,
    schedule: Schedule,
    trace: Trace,
    replaying: Option<Replaying>,
    /// The allocations `gc=` drew for.
    allocations: u64,
    /// The pinned waits begun.
    pinned_waits: u64,
    /// Whether a wait nothing in the simulation can end was refused.
    stuck: bool,
    /// What another thread posted, where it posted something other than a
    /// signal's halt.
    breach: Option<&'static str>,
    /// Whether an inline native call outlasted `block=`.
    blocked_native: bool,
    /// The first invariant a switch found broken, until a boundary refuses
    /// it.
    inconsistent: Option<Loud>,
    /// Whether a switch found an invariant broken.
    invariant_broken: bool,
}

/// The watch on an inline native call in the simulation mode: a thread that
/// interrupts the interpreter's thread once, if the call outlasts `block=`.
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

    /// Enters the simulation mode for the rest of the run: `pre:` and `pct:`
    /// draw their points, and `pct:` the priorities of the activities there
    /// are.
    pub(crate) fn start_sim(&mut self, config: SimConfig) {
        let mut streams = Streams::new(config.seed);
        let clock = Clock::new(config.knobs.clock, &mut streams.clock);
        let points = match config.policy {
            Policy::Pre { d, k } => draw_points(&mut streams.schedule, u64::from(d), k),
            Policy::Pct { d, k } => draw_points(&mut streams.schedule, u64::from(d) - 1, k),
            Policy::Fifo | Policy::Uniform { .. } => Vec::new(),
        };
        let replaying = config.replay.as_ref().map(Replaying::new);
        let collecting = config.knobs.gc.is_some();
        self.sim = Some(Box::new(Sim {
            trace: Trace::new(config.trace.is_some()),
            config,
            streams,
            clock,
            schedule: Schedule {
                contended: 0,
                run_contended: 0,
                seen_switches: self.activities.switches(),
                points,
                next_point: 0,
                changes: 0,
                priorities: Vec::new(),
            },
            replaying,
            allocations: 0,
            pinned_waits: 0,
            stuck: false,
            breach: None,
            blocked_native: false,
            inconsistent: None,
            invariant_broken: false,
        }));
        for handle in 0..self.activities.handles() {
            self.sim_spawned(handle);
        }
        self.pool.set_bound(0);
        self.timer.requests().set(crate::timer::SIM);
        self.activity.random_source = None;
        if collecting {
            self.collect_due = true;
        }
    }

    /// Gives the activity under `handle` its seeded priority, under `pct:`.
    pub(crate) fn sim_spawned(&mut self, handle: usize) {
        let Some(sim) = self.sim.as_deref_mut() else {
            return;
        };
        if !matches!(sim.config.policy, Policy::Pct { .. }) {
            return;
        }
        let priorities = &mut sim.schedule.priorities;
        if priorities.len() <= handle {
            priorities.resize(handle + 1, PRIORITY_BASE);
        }
        priorities[handle] =
            PRIORITY_BASE + sim.streams.schedule.within(0..=u64::from(u32::MAX)) as i64;
    }

    /// A clause boundary in the simulation mode, the `clauses`-th: virtual
    /// time moves on a quantum, `halt@K` halts at its boundary, and where
    /// another activity is ready the step is contended and the policy
    /// answers whether the running activity is preempted.
    #[cold]
    #[inline(never)]
    pub(crate) fn sim_clause(&mut self, clauses: u64) -> Result<bool, Failure> {
        let Some(sim) = self.sim.as_deref_mut() else {
            return Ok(false);
        };
        sim.clock.elapsed += sim.clock.quantum;
        let halt = sim.config.knobs.halt_at == Some(clauses);
        self.sim_breached()?;
        if halt {
            self.halt_all();
        }
        if self.activities.ready.is_empty() {
            return Ok(false);
        }
        Ok(self.sim_preempts())
    }

    /// The policy's answer at a contended step, recorded where it preempts.
    fn sim_preempts(&mut self) -> bool {
        let switches = self.activities.switches();
        let running = self.activities.running_index();
        let ready = &self.activities.ready;
        let Some(sim) = self.sim.as_deref_mut() else {
            return false;
        };
        let schedule = &mut sim.schedule;
        schedule.contended += 1;
        let step = schedule.contended;
        if schedule.seen_switches != switches {
            schedule.seen_switches = switches;
            schedule.run_contended = 0;
        }
        schedule.run_contended += 1;
        let forced = schedule.run_contended >= sim.config.knobs.floor;
        if forced {
            schedule.run_contended = 0;
        }
        let chosen = match (&mut sim.replaying, sim.config.policy) {
            (Some(replaying), _) => replaying.point(Point::Preempt, step, forced),
            (None, Policy::Fifo) => false,
            (None, Policy::Pre { .. }) => reached(&schedule.points, &mut schedule.next_point, step),
            (None, Policy::Uniform { p }) => sim.streams.schedule.unit() < p,
            (None, Policy::Pct { d, .. }) => {
                let priority = |handle: usize, priorities: &[i64]| {
                    priorities.get(handle).copied().unwrap_or(PRIORITY_BASE)
                };
                if reached(&schedule.points, &mut schedule.next_point, step) {
                    schedule.changes += 1;
                    set_priority(
                        &mut schedule.priorities,
                        running,
                        i64::from(d) - schedule.changes,
                    );
                }
                if forced {
                    let lowest = schedule.priorities.iter().copied().min().unwrap_or(0);
                    set_priority(&mut schedule.priorities, running, lowest - 1);
                }
                let mine = priority(running, &schedule.priorities);
                ready
                    .iter()
                    .any(|other| priority(other.index(), &schedule.priorities) > mine)
            }
        };
        let preempt = chosen || forced;
        if preempt {
            sim.trace.record(Decision::Preempt(step));
        }
        preempt
    }

    /// The ready activity the next switch takes: under `pct:` the first of
    /// the highest priority, else the front.
    pub(crate) fn sim_pick(&mut self) -> Option<crate::scheduler::ActivityId> {
        let ready = &mut self.activities.ready;
        let Some(sim) = self.sim.as_deref_mut() else {
            return ready.pop_front();
        };
        if ready.is_empty() || !matches!(sim.config.policy, Policy::Pct { .. }) {
            return ready.pop_front();
        }
        let last = ready.len() as u64 - 1;
        let index = match &mut sim.replaying {
            Some(replaying) => replaying.pick(last),
            None => {
                let priorities = &sim.schedule.priorities;
                let priority =
                    |handle: usize| priorities.get(handle).copied().unwrap_or(PRIORITY_BASE);
                let mut best = 0;
                for (at, activity) in ready.iter().enumerate() {
                    if priority(activity.index()) > priority(ready[best].index()) {
                        best = at;
                    }
                }
                best as u64
            }
        };
        sim.trace.record(Decision::Pick(index));
        ready.remove(index as usize)
    }

    /// Shuffles the activities one event readied, those from `mark` on in
    /// the ready queue, unless `order=fifo` or `pct:` orders them.
    #[cold]
    #[inline(never)]
    pub(crate) fn sim_order_event(&mut self, mark: usize) {
        let ready = &mut self.activities.ready;
        let Some(sim) = self.sim.as_deref_mut() else {
            return;
        };
        if sim.config.order == Order::Fifo || matches!(sim.config.policy, Policy::Pct { .. }) {
            return;
        }
        let readied = ready.len().saturating_sub(mark);
        for i in (1..readied).rev() {
            let j = match &mut sim.replaying {
                Some(replaying) => replaying.pick(i as u64),
                None => sim.streams.order.within(0..=i as u64),
            };
            sim.trace.record(Decision::Pick(j));
            ready.swap(mark + i, mark + j as usize);
        }
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
        if let Some(loud) = self
            .sim
            .as_deref_mut()
            .and_then(|sim| sim.inconsistent.take())
        {
            return Err(loud.into());
        }
        if let Some(loud) = self.sim_replay_refusal() {
            return Err(loud.into());
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

    /// Whether a wait nothing in the simulation can end was refused, or a
    /// replay diverged, which ends the program's wait for its activities.
    pub(crate) fn sim_is_stuck(&self) -> bool {
        self.sim.as_ref().is_some_and(|sim| {
            sim.stuck || sim.replaying.as_ref().is_some_and(|replay| replay.diverged)
        })
    }

    /// The refusal of a replay's divergence, once.
    fn sim_replay_refusal(&mut self) -> Option<Loud> {
        let sim = self.sim.as_deref_mut()?;
        sim.replaying.as_mut()?.refusal.take()
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
    #[cold]
    #[inline(never)]
    pub(crate) fn sim_declines_collection(&mut self) -> bool {
        let Some(sim) = self.sim.as_deref_mut() else {
            return false;
        };
        let Some(q) = sim.config.knobs.gc else {
            return false;
        };
        sim.allocations += 1;
        let at = sim.allocations;
        let collects = match &mut sim.replaying {
            Some(replaying) => replaying.point(Point::Collect, at, false),
            None => sim.streams.gc.unit() < q,
        };
        if collects {
            sim.trace.record(Decision::Collect(at));
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
            let timed_out = matches!(
                stopped.recv_timeout(bound),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            );
            if timed_out {
                target.interrupt();
                let _ = stopped.recv();
            }
            timed_out
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
            contended: sim.schedule.contended,
            switches: self.activities.switches(),
            trace_hash: Some(sim.trace.hash),
        })
    }

    /// The refusals a run in the simulation mode ends with: the one a
    /// boundary has not raised yet, a replay's decisions left, and a trace
    /// file not written. The decision trace goes to `trace=`'s file: its
    /// header, `CONFIG hash=H`, then one decision a line.
    pub(crate) fn sim_finish(&mut self) -> Vec<Loud> {
        let mut refused = Vec::new();
        let Some(sim) = self.sim.as_deref_mut() else {
            return refused;
        };
        if let Some(replaying) = &mut sim.replaying {
            replaying.finish();
        }
        if let Err(Failure::Loud(loud)) = self.sim_breached() {
            refused.push(*loud);
        }
        refused.extend(self.sim_replay_refusal());
        let Some(sim) = self.sim.as_deref() else {
            return refused;
        };
        let (Some(path), Some(kept)) = (&sim.config.trace, &sim.trace.kept) else {
            return refused;
        };
        let mut text = format!("{} hash={:016x}\n", sim.config.header(), sim.trace.hash);
        for decision in kept {
            text.push_str(&format!("{decision}\n"));
        }
        if let Err(error) = std::fs::write(path, text) {
            refused.push(Loud::sim_trace_unwritten(path, &error));
        }
        refused
    }

    /// Checks the scheduler's invariants at a switch; the first broken one is
    /// refused at the next boundary, and ends the program's wait for its
    /// activities. No switch after it is checked.
    #[cold]
    #[inline(never)]
    pub(crate) fn sim_check_switch(&mut self) {
        if self.sim.as_deref().is_some_and(|sim| sim.invariant_broken) {
            return;
        }
        #[cfg(test)]
        self.sim_corrupt();
        let checked = self.check_invariants();
        #[cfg(test)]
        self.sim_uncorrupt();
        let Err(loud) = checked else {
            return;
        };
        if let Some(sim) = self.sim.as_deref_mut() {
            sim.stuck = true;
            sim.invariant_broken = true;
            sim.inconsistent = Some(loud);
        }
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
    /// `block=`'s real time.
    pub(crate) fn sim_blocked_native() -> Loud {
        Loud {
            message: crate::owned_message(
                "a native call in the simulation mode that runs longer than its bound",
                None,
            ),
        }
    }

    /// A replay that took a decision its trace file does not hold; `what`
    /// names the file, the decision and what the run did.
    pub(crate) fn sim_replay_diverged(what: String) -> Loud {
        Loud { message: what }
    }

    /// A decision trace that could not be written to `path`.
    pub(crate) fn sim_trace_unwritten(path: &Path, error: &std::io::Error) -> Loud {
        Loud {
            message: format!(
                "the trace `{}` could not be written: {error}",
                path.display()
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
