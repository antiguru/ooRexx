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

use crate::{Invocation, Outcome, SimConfig, SwitchMode, run_program};

/// A bound on every run here, real time.
const RUN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(60);

fn sim(spec: &str) -> SwitchMode {
    SwitchMode::Sim(SimConfig::parse(spec).expect("a sim spec"))
}

fn run_in(source: &str, mode: Option<SwitchMode>) -> Outcome {
    let invocation = Invocation::none().with_deadline(RUN_DEADLINE);
    let invocation = match mode {
        Some(mode) => invocation.with_switch_mode(mode),
        None => invocation,
    };
    run_program("/tmp/sim.rex", source.as_bytes().to_vec(), invocation)
}

fn stdout(outcome: &Outcome) -> String {
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

fn stderr(outcome: &Outcome) -> String {
    String::from_utf8_lossy(&outcome.stderr).into_owned()
}

/// Sleeps 5 s, reads `time('e')`, lets an alarm ring and times out a wait on
/// an event semaphore: 8 s of program time, each value read as a predicate.
/// The oracle prints these four lines in 8.01 s.
const TIMED: &str = "call time 'R'\n\
    call SysSleep 5\n\
    e = time('E')\n\
    say 'slept' (e >= 5) (e < 6)\n\
    alarm = .Alarm~new(1, .Message~new(.ringer, 'RING'))\n\
    call SysSleep 2\n\
    say 'rang' .ringer~rung\n\
    sem = .EventSemaphore~new\n\
    say 'waited' sem~wait(1)\n\
    e = time('E')\n\
    say 'elapsed' (e >= 8) (e < 9)\n\
    ::class ringer\n\
    ::attribute rung class\n\
    ::method init class\n  self~rung = 0\n\
    ::method ring class\n  self~rung = 1\n";

const TIMED_OUT: &str = "slept 1 1\nrang 1\nwaited 0\nelapsed 1 1\n";

#[test]
fn a_timed_program_runs_on_virtual_time_and_prints_what_it_prints_on_real_time() {
    let began = std::time::Instant::now();
    let simulated = run_in(TIMED, Some(sim("sim:1")));
    let took = began.elapsed();
    assert_eq!(simulated.exit_code, 0, "{}", stderr(&simulated));
    assert_eq!(stdout(&simulated), TIMED_OUT);
    assert!(took < std::time::Duration::from_secs(1), "took {took:?}");
    let real = run_in(TIMED, None);
    assert_eq!(real.exit_code, 0, "{}", stderr(&real));
    assert_eq!(stdout(&real), TIMED_OUT);
}

/// Unseeded `RANDOM`, `TIME` and `DATE`, which read the seed's streams and
/// clock.
const DRAWN: &str = "say random() random() random(1, 1000000)\n\
    say time('L') date('S') time('E')\n\
    t = .w~new~start('DRAW')\n\
    say t~result\n\
    ::class w\n::method draw\n  return random() time('L')\n";

#[test]
fn one_seed_gives_one_output() {
    let first = run_in(DRAWN, Some(sim("sim:1")));
    let second = run_in(DRAWN, Some(sim("sim:1")));
    assert_eq!(first.exit_code, 0, "{}", stderr(&first));
    assert_eq!(stdout(&first), stdout(&second));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn two_seeds_draw_different_randoms() {
    let one = run_in("say random() random() random()\n", Some(sim("sim:1")));
    let two = run_in("say random() random() random()\n", Some(sim("sim:2")));
    assert_eq!(one.exit_code, 0, "{}", stderr(&one));
    assert_eq!(two.exit_code, 0, "{}", stderr(&two));
    assert_ne!(stdout(&one), stdout(&two));
}

#[test]
fn a_config_reads_back_what_it_prints() {
    for text in [
        "sim:7,fifo",
        "sim:18446744073709551615,fifo,gc=0.25,halt@3,fail=wait:2,block=0.5,clock=midnight",
        "sim:0,fifo,clock=real",
        "sim:7,pre:2,k=40",
        "sim:7,uniform:0.2,order=fifo",
        "sim:7,pct:3,k=40,gc=0.5,floor=1000,trace=/tmp/x",
    ] {
        let config = SimConfig::parse(text).expect("a sim spec");
        assert_eq!(config.to_string(), text);
    }
    assert_eq!(
        SimConfig::parse("sim:7").map(|c| c.to_string()),
        Ok("sim:7,fifo".into())
    );
    assert_eq!(
        SimConfig::parse("sim:7,block=0.001").map(|c| c.to_string()),
        Ok("sim:7,fifo,block=0.001".into())
    );
    assert_eq!(
        SimConfig::parse("sim:7,block=1e30"),
        Err("`block=1e30`: block is a number of seconds from 0.001 to 86400".into())
    );
    for wrong in [
        "sim:",
        "sim:x",
        "sim:1,gc=2",
        "sim:1,halt@0",
        "sim:1,fail=wait:",
        "sim:1,block=-1",
        "sim:1,block=0",
        "sim:1,block=0.0009",
        "sim:1,block=x",
        "sim:1,block=inf",
        "sim:1,block=NaN",
        "sim:1,block=1e30",
        "sim:1,pre:1",
        "sim:1,pre:0,k=3",
        "sim:1,pct:1",
        "sim:1,pct:1,k=0",
        "sim:1,uniform:2",
        "sim:1,k=3",
        "sim:1,floor=0",
        "sim:1,order=one",
        "sim:replay=/nonexistent/trace",
        "simx",
    ] {
        assert!(SimConfig::parse(wrong).is_err(), "{wrong} parsed");
    }
}

/// An empty path, a second `trace=` or `replay=`, and a path cut at a comma
/// are refused, each naming what it refuses.
#[test]
fn a_trace_path_is_refused_where_it_cannot_be_the_file_meant() {
    for (wrong, refusal) in [
        ("sim:1,trace=", "`trace=`: an empty path"),
        ("sim:replay=", "`replay=`: an empty path"),
        ("sim:replay=/t,trace=", "`trace=`: an empty path"),
        ("sim:1,trace=/a,trace=/b", "`trace=/b`: a second `trace=`"),
        (
            "sim:replay=/a,trace=/b,trace=/c",
            "`trace=/c`: a second `trace=`",
        ),
        ("sim:replay=/a,replay=/b", "`replay=/b`: a second `replay=`"),
        (
            "sim:1,trace=/tmp/a,b.txt",
            "`trace=/tmp/a,b.txt`: a path may not contain a comma",
        ),
        (
            "sim:replay=/tmp/c,d.txt",
            "`replay=/tmp/c,d.txt`: a path may not contain a comma",
        ),
        (
            "sim:replay=/tmp/c,trace=/tmp/e,f",
            "`trace=/tmp/e,f`: a path may not contain a comma",
        ),
    ] {
        assert_eq!(SimConfig::parse(wrong), Err(refusal.into()), "{wrong}");
    }
}

/// splitmix64 from 0, and xoshiro256** from `[1, 2, 3, 4]`, against their
/// reference outputs.
#[test]
fn the_generators_give_their_reference_outputs() {
    let mut state = 0;
    assert_eq!(super::splitmix64(&mut state), 0xE220_A839_7B1D_CDAF);
    let mut rng = super::Rng([1, 2, 3, 4]);
    let drawn: Vec<u64> = (0..4).map(|_| rng.next_u64()).collect();
    assert_eq!(drawn, [11520, 0, 1_509_978_240, 1_215_971_899_390_074_240]);
}

/// A started activity whose wait nothing can end is refused at the
/// program's end, where real time waits for the run's deadline.
#[test]
fn a_wait_only_a_signal_can_end_is_refused() {
    let source = ".w~new~start('WAITON', .message~new('abc', 'length'))\nsay 'main done'\n\
                  ::class w\n::method waiton\n  use arg m\n  say 'waits'\n  return m~result\n";
    let outcome = run_in(source, Some(sim("sim:1")));
    assert_eq!(outcome.exit_code, 120);
    assert_eq!(stdout(&outcome), "main done\nwaits\n");
    assert_eq!(
        stderr(&outcome),
        "rexx-exec: a wait in the simulation mode that only a signal can end is not implemented\n"
    );
    let real = run_program(
        "/tmp/sim.rex",
        source.as_bytes().to_vec(),
        Invocation::none().with_deadline(std::time::Duration::from_millis(300)),
    );
    assert_eq!(real.exit_code, crate::DEADLINE_EXIT);
}

/// `halt@K` halts every activity at clause boundary `K`, the same boundary
/// on every run.
#[test]
fn halt_at_k_halts_at_its_boundary() {
    let source = "signal on halt\ndo i = 1 to 1000\nend\nsay 'done'\nexit\nhalt:\nsay 'halted' i\n";
    let first = run_in(source, Some(sim("sim:1,halt@50")));
    let second = run_in(source, Some(sim("sim:9,halt@50")));
    assert_eq!(first.exit_code, 0, "{}", stderr(&first));
    assert!(stdout(&first).starts_with("halted "), "{}", stdout(&first));
    assert_eq!(stdout(&first), stdout(&second));
    let later = run_in(source, Some(sim("sim:1,halt@60")));
    assert_ne!(stdout(&first), stdout(&later));
    let unhalted = run_in(source, Some(sim("sim:1")));
    assert_eq!(stdout(&unhalted), "done\n");
}

/// The pinned wait a sort comparator makes on a message's result: `fail=wait:1`
/// fails it with 11.1, without the knob it answers.
const PINNED: &str = "signal on syntax\nm = .message~new('abc', 'length')\n\
     t = .k~new~start('SEND', m)\n\
     a = .array~of(2, 1)\na~sortWith(.c~new(m))\nsay 'sorted' a~toString('L', ',')\nexit\n\
     syntax:\nsay 'trapped' condition('o')~code\nexit\n\
     ::class k\n::method send\n  use arg m\n  m~send\n\
     ::class c\n::method init\n  expose m\n  use arg m\n\
     ::method compare\n  expose m\n  use arg l, r\n  say m~result\n  return l - r\n";

#[test]
fn fail_wait_k_fails_the_kth_pinned_wait() {
    let failed = run_in(PINNED, Some(sim("sim:1,fail=wait:1")));
    assert_eq!(failed.exit_code, 0, "{}", stderr(&failed));
    assert_eq!(stdout(&failed), "trapped 11.1\n");
    let second = run_in(PINNED, Some(sim("sim:1,fail=wait:2")));
    assert_eq!(stdout(&second), "3\nsorted 1,2\n");
    let waited = run_in(PINNED, Some(sim("sim:1")));
    assert_eq!(stdout(&waited), "3\nsorted 1,2\n");
}

/// Allocates enough short-lived strings, each too long to be held inline,
/// that a collection at each allocation with probability 0.01 happens, and
/// few enough that no other trigger fires.
const ALLOCATING: &str = "do i = 1 to 3000\n  s = 'a long string' || i\nend\nsay 'done'\n";

#[test]
fn gc_q_collects_at_random_allocations_the_seed_chooses() {
    let none = run_in(ALLOCATING, Some(sim("sim:1")));
    assert_eq!(none.collections, 0);
    let first = run_in(ALLOCATING, Some(sim("sim:1,gc=0.01")));
    let again = run_in(ALLOCATING, Some(sim("sim:1,gc=0.01")));
    assert_eq!(stdout(&first), "done\n");
    assert!(first.collections > 0);
    assert_eq!(first.collections, again.collections);
    let zero = run_in(ALLOCATING, Some(sim("sim:1,gc=0")));
    assert_eq!(zero.collections, 0);
}

/// The gc knob draws from its own stream: random numbers are the same with
/// it on and off.
#[test]
fn a_knob_leaves_the_other_streams_draws() {
    let source = "do i = 1 to 300\n  s = 'x' || i\nend\nsay random() random() time('L')\n";
    let plain = run_in(source, Some(sim("sim:4")));
    let collecting = run_in(source, Some(sim("sim:4,gc=0.5")));
    assert!(collecting.collections > 0);
    assert_eq!(stdout(&plain), stdout(&collecting));
}

/// A command's child gets a seed of its own, drawn from the children
/// stream, and the parent's policy and knobs but `halt@` and `fail=`.
#[test]
fn a_child_gets_a_seed_from_the_children_stream() {
    let source =
        "address system 'echo $REXX_SWITCH_MODE'\naddress system 'echo $REXX_SWITCH_MODE'\n";
    let first = run_in(source, Some(sim("sim:1,gc=0.5,halt@1000")));
    let again = run_in(source, Some(sim("sim:1,gc=0.5,halt@1000")));
    assert_eq!(first.exit_code, 0, "{}", stderr(&first));
    assert_eq!(stdout(&first), stdout(&again));
    let modes: Vec<&str> = stdout(&first).leak().lines().collect();
    assert_eq!(modes.len(), 2);
    assert_ne!(modes[0], modes[1]);
    for mode in modes {
        let child = SimConfig::parse(mode).expect("the child's mode parses");
        assert_ne!(child.seed, 1);
        assert_eq!(mode, format!("sim:{},fifo,gc=0.5", child.seed));
    }
}

/// `clock=midnight` starts seconds before a local midnight, so a sleep
/// crosses into the next day.
#[test]
fn clock_midnight_crosses_a_day_boundary() {
    let source = "d = date('S')\nt = time('S')\ncall SysSleep 6\n\
                  say (t >= 86394) (date('B', date('S'), 'S') - date('B', d, 'S'))\n";
    let outcome = run_in(source, Some(sim("sim:3,clock=midnight")));
    assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
    assert_eq!(stdout(&outcome), "1 1\n");
}

/// A post from another thread is a determinism breach, refused at the next
/// clause boundary; a signal's halt is not.
#[test]
fn a_post_from_another_thread_is_refused() {
    let post = |posted: crate::scheduler::Posted| {
        let mut interp = crate::Interp::new();
        interp.set_switch_mode(sim("sim:1"));
        let inbox = interp.timer.inbox();
        std::thread::spawn(move || inbox.post(posted))
            .join()
            .expect("the posting thread");
        interp
            .serve_requests(true)
            .err()
            .map(|failure| match failure {
                crate::Failure::Loud(loud) => loud.message,
                _ => panic!("a failure that is not a refusal"),
            })
    };
    assert_eq!(
        post(crate::scheduler::Posted::Output {
            error: false,
            bytes: b"x".to_vec(),
        }),
        Some(
            "a command's output from another thread in the simulation mode is not implemented"
                .into()
        )
    );
    assert_eq!(post(crate::scheduler::Posted::Halt), None);
}

/// Main reads a fifo with a command another activity's command writes. On
/// real time the reader's wait leaves the baton and the writer runs; in the
/// simulation mode the wait is on the baton, and `block=`'s real-time bound
/// abandons and refuses it.
#[test]
fn a_command_only_another_activity_can_end_is_refused_after_its_bound() {
    let fifo = std::env::temp_dir().join(format!("rexx-sim-fifo-{}", std::process::id()));
    let made = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("mkfifo runs");
    assert!(made.success());
    let fifo_text = fifo.to_string_lossy().into_owned();
    let source = format!(
        "w = .w~new~start('WRITE')\naddress system 'cat {fifo_text}'\nsay 'read done' w~result\n\
         ::class w\n::method write\n  call SysSleep 0.1\n  \
         address system 'echo hi > {fifo_text}'\n  return 'wrote'\n"
    );
    let real = run_in(&source, None);
    assert_eq!(real.exit_code, 0, "{}", stderr(&real));
    assert_eq!(stdout(&real), "hi\nread done wrote\n");
    let began = std::time::Instant::now();
    let simulated = run_in(&source, Some(sim("sim:1,block=0.5")));
    let took = began.elapsed();
    // Ends a reader the kill left behind, if any.
    {
        use std::os::unix::fs::OpenOptionsExt;
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&fifo);
    }
    let _ = std::fs::remove_file(&fifo);
    assert_eq!(simulated.exit_code, 120);
    assert_eq!(stdout(&simulated), "");
    assert_eq!(
        stderr(&simulated),
        "rexx-exec: a command in the simulation mode that waits longer than its bound is not \
         implemented\n"
    );
    assert!(
        took >= std::time::Duration::from_millis(500),
        "took {took:?}"
    );
    assert!(took < std::time::Duration::from_secs(5), "took {took:?}");
}

/// Main prints two lines while a started activity waits to print one: with
/// no preemption the started line comes last, and one preemption at a
/// contended step drawn among the first three puts it earlier.
const TWO_ACTIVITIES: &str = "t = .t~new~start('run')\nsay 'm1'\nsay 'm2'\nt~wait\n\
     ::class t\n::method run\n  say 't'\n";

#[test]
fn pre_1_reaches_every_interleaving_over_a_seed_range() {
    let unpreempted = run_in(TWO_ACTIVITIES, Some(sim("sim:1")));
    assert_eq!(stdout(&unpreempted), "m1\nm2\nt\n");
    let mut seen = std::collections::BTreeSet::new();
    for seed in 1..=40 {
        let outcome = run_in(TWO_ACTIVITIES, Some(sim(&format!("sim:{seed},pre:1,k=3"))));
        assert_eq!(outcome.exit_code, 0, "{}", stderr(&outcome));
        seen.insert(stdout(&outcome));
    }
    let every: std::collections::BTreeSet<String> = ["t\nm1\nm2\n", "m1\nt\nm2\n", "m1\nm2\nt\n"]
        .map(String::from)
        .into();
    assert_eq!(seen, every);
}

/// A started activity polls a flag with no park point in its loop while the
/// activity that sets it is ready.
const POLLING: &str = "f = .flag~new\na = f~start('waitForFlag')\nf~start('setFlag')\n\
     say a~result\n::class flag\n::attribute done unguarded\n::method init\n  expose done\n  \
     done = 0\n::method waitForFlag unguarded\n  do while \\self~done\n  end\n  \
     return 'A ended'\n::method setFlag unguarded\n  self~done = 1\n";

/// No policy below preempts the poller in time, so each run ends because
/// the fairness floor forced a preemption, after at least that many steps.
#[test]
fn a_polling_activity_ends_by_the_floor_under_every_policy() {
    for policy in ["fifo", "pre:1,k=1", "uniform:0", "pct:1,k=1"] {
        let spec = format!("sim:5,{policy}");
        let outcome = run_in(POLLING, Some(sim(&spec)));
        assert_eq!(outcome.exit_code, 0, "{spec}: {}", stderr(&outcome));
        assert_eq!(stdout(&outcome), "A ended\n", "{spec}");
        let report = outcome.sim.as_ref().expect("a report");
        assert!(report.steps >= super::FAIRNESS_FLOOR, "{spec}: {report:?}");
    }
    let small = run_in(POLLING, Some(sim("sim:5,floor=50")));
    assert_eq!(stdout(&small), "A ended\n");
    let report = small.sim.as_ref().expect("a report");
    assert!(report.steps < 1000, "{report:?}");
}

/// Waiters one completion wakes, two activities preempted at random and
/// collections at random: every decision kind, written and replayed.
const DECIDING: &str = "m = .message~new('abc', 'length')\n\
     a = .w~new~start('WAITON', m, 'a')\nb = .w~new~start('WAITON', m, 'b')\n\
     c = .w~new~start('WAITON', m, 'c')\nd = .w~new~start('SEND', m)\n\
     do i = 1 to 50\n  s = 'a long string' i\nend\n\
     say a~result b~result c~result d~result\n\
     ::class w\n::method waiton\n  use arg m, name\n  say name 'waits'\n  r = m~result\n  \
     say name 'woke'\n  do i = 1 to 20\n    s = 'another long string' i\n  end\n  return r\n\
     ::method send\n  use arg m\n  m~send\n  say 'sent'\n  return 'd'\n";

#[test]
fn a_recorded_trace_replays_to_the_same_output_and_hash() {
    let dir = std::env::temp_dir().join(format!("rexx-sim-trace-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    let file = dir.join("trace.txt");
    let spec = format!("sim:3,uniform:0.2,gc=0.05,trace={}", file.display());
    let recorded = run_in(DECIDING, Some(sim(&spec)));
    assert_eq!(recorded.exit_code, 0, "{}", stderr(&recorded));
    let written = std::fs::read_to_string(&file).expect("the trace written");
    let mut lines = written.lines();
    let hash = |outcome: &Outcome| outcome.sim.as_ref().and_then(|sim| sim.trace_hash);
    let header = format!(
        "sim:3,uniform:0.2,gc=0.05 hash={:016x}",
        hash(&recorded).expect("a hash")
    );
    assert_eq!(lines.next(), Some(header.as_str()));
    for kind in ["preempt ", "pick ", "collect "] {
        assert!(written.contains(kind), "no {kind}in {written}");
    }
    let replayed = run_in(
        DECIDING,
        Some(sim(&format!("sim:replay={}", file.display()))),
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        (replayed.exit_code, stdout(&replayed), stderr(&replayed)),
        (recorded.exit_code, stdout(&recorded), stderr(&recorded))
    );
    assert_eq!(recorded.collections, replayed.collections);
    assert_eq!(hash(&recorded), hash(&replayed));
    let other = run_in(DECIDING, Some(sim("sim:4,uniform:0.2,gc=0.05")));
    assert_ne!(hash(&recorded), hash(&other));
}

/// A directory of its own for a test's trace files.
fn trace_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("rexx-sim-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    dir
}

/// Records `DECIDING` under `sim:3,uniform:0.3` to `file`, and answers the
/// decision lines.
fn record_deciding(file: &std::path::Path) -> Vec<String> {
    let recorded = run_in(
        DECIDING,
        Some(sim(&format!("sim:3,uniform:0.3,trace={}", file.display()))),
    );
    assert_eq!(recorded.exit_code, 0, "{}", stderr(&recorded));
    let written = std::fs::read_to_string(file).expect("the trace written");
    written.lines().skip(1).map(String::from).collect()
}

/// Writes `decisions` to `file` under `DECIDING`'s header with their own hash,
/// so that the file loads and only the run can tell it from the recording.
fn write_rehashed(file: &std::path::Path, decisions: &[String]) {
    let parsed: Vec<super::Decision> = decisions
        .iter()
        .map(|line| {
            let (kind, value) = line.split_once(' ').expect("a decision");
            let value = value.parse().expect("a value");
            match kind {
                "preempt" => super::Decision::Preempt(value),
                "pick" => super::Decision::Pick(value),
                _ => super::Decision::Collect(value),
            }
        })
        .collect();
    let mut text = format!(
        "sim:3,uniform:0.3 hash={:016x}\n",
        super::trace_hash(&parsed)
    );
    for line in decisions {
        text.push_str(line);
        text.push('\n');
    }
    std::fs::write(file, text).expect("the trace rewritten");
}

/// Replays `file` on `source`.
fn replay(source: &str, file: &std::path::Path) -> Outcome {
    run_in(source, Some(sim(&format!("sim:replay={}", file.display()))))
}

/// A replay that takes a decision its file does not hold, or ends with
/// decisions left, is refused naming the file, the decision's place and
/// what the run did; the same file replays on its own program.
#[test]
fn a_replay_that_diverges_is_refused() {
    let dir = trace_dir("diverge");
    let file = dir.join("t.txt");
    let decisions = record_deciding(&file);
    let path = file.display().to_string();
    let same = replay(DECIDING, &file);
    assert_eq!(same.exit_code, 0, "{}", stderr(&same));
    let refusal = |at: usize, held: &str, did: &str| {
        format!(
            "rexx-exec: the replay of `{path}` diverged at decision {at}: the file holds {held}, \
             the run {did}\n"
        )
    };
    let other = replay(THREE_ACTIVITIES, &file);
    let first_pick = decisions
        .iter()
        .position(|line| line.starts_with("pick "))
        .expect("a pick");
    let out_of_range = {
        let mut edited = decisions.clone();
        edited[first_pick] = "pick 999".into();
        write_rehashed(&file, &edited);
        replay(DECIDING, &file)
    };
    let truncated = {
        write_rehashed(&file, &decisions[..first_pick]);
        replay(DECIDING, &file)
    };
    let extra = {
        let mut edited = decisions.clone();
        edited.push("preempt 999999".into());
        write_rehashed(&file, &edited);
        replay(DECIDING, &file)
    };
    let kind = {
        let mut edited = decisions.clone();
        edited.insert(0, "pick 0".into());
        write_rehashed(&file, &edited);
        replay(DECIDING, &file)
    };
    let passed = {
        let mut edited = decisions.clone();
        edited.swap(0, 1);
        write_rehashed(&file, &edited);
        replay(DECIDING, &file)
    };
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(decisions[..2], ["preempt 3", "preempt 7"]);
    for (name, outcome, expected) in [
        ("other", &other, refusal(4, "`preempt 16`", "ends")),
        (
            "out of range",
            &out_of_range,
            refusal(first_pick + 1, "`pick 999`", "picks from 0 to 2"),
        ),
        (
            "truncated",
            &truncated,
            refusal(first_pick + 1, "no more", "picks from 0 to 2"),
        ),
        (
            "extra",
            &extra,
            refusal(decisions.len() + 1, "`preempt 999999`", "ends"),
        ),
        (
            "kind",
            &kind,
            refusal(1, "`pick 0`", "preempts at contended step 3"),
        ),
        (
            "passed",
            &passed,
            refusal(
                2,
                "`preempt 3`",
                "reaches contended step 8 without preempting at 3",
            ),
        ),
    ] {
        assert_eq!(
            (outcome.exit_code, stderr(outcome)),
            (120, expected),
            "{name}"
        );
    }
}

/// A trace whose decisions do not hash to its header's hash is refused
/// before the run, and so is a header without one; the file as written
/// loads.
#[test]
fn a_damaged_trace_is_refused_before_the_run() {
    let dir = trace_dir("damaged");
    let file = dir.join("t.txt");
    let decisions = record_deciding(&file);
    let written = std::fs::read_to_string(&file).expect("the trace written");
    let loads = SimConfig::parse(&format!("sim:replay={}", file.display())).is_ok();
    let (header, rest) = written.split_once('\n').expect("a header");
    let (configuration, hash) = header.rsplit_once(" hash=").expect("a hash");
    let damaged = format!("{configuration} hash=0123456789abcdef\n{rest}");
    std::fs::write(&file, damaged).expect("the trace damaged");
    let wrong_hash = SimConfig::parse(&format!("sim:replay={}", file.display()));
    std::fs::write(&file, format!("{configuration}\n{rest}")).expect("the hash dropped");
    let no_hash = SimConfig::parse(&format!("sim:replay={}", file.display()));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(loads);
    assert!(!decisions.is_empty());
    let path = file.display();
    assert_eq!(
        wrong_hash,
        Err(format!(
            "`{path}`: the decisions hash to {hash}, the header holds 0123456789abcdef"
        ))
    );
    assert_eq!(
        no_hash,
        Err(format!(
            "`{path}`: `{configuration}` is not a trace's header, `CONFIG hash=H`"
        ))
    );
}

/// A trace file that cannot be written is refused, and the run's status
/// says so.
#[test]
fn a_trace_that_cannot_be_written_is_refused() {
    let outcome = run_in(
        "say 'ran'\n",
        Some(sim("sim:1,trace=/nonexistent/dir/t.txt")),
    );
    assert_eq!(
        (outcome.exit_code, stdout(&outcome), stderr(&outcome)),
        (
            120,
            "ran\n".into(),
            "rexx-exec: the trace `/nonexistent/dir/t.txt` could not be written: No such file or \
             directory (os error 2)\n"
                .into()
        )
    );
}

/// Runs `source` under `spec` with `corruption` asked of the first switch
/// that has what it breaks.
fn corrupted(
    source: &'static str,
    spec: &str,
    corruption: Option<crate::scheduler::Corruption>,
) -> Outcome {
    let invocation = Invocation::none()
        .with_deadline(RUN_DEADLINE)
        .with_switch_mode(sim(spec));
    crate::on_interpreter_thread(move || {
        crate::scheduler::CORRUPTION.with(|asked| asked.set(corruption));
        crate::execute_on(
            "/tmp/sim.rex",
            source.as_bytes().to_vec(),
            false,
            invocation,
            Some(crate::INTERPRETER_STACK_BYTES),
        )
    })
}

/// Two started activities, so a switch to one leaves the other ready.
const THREE_ACTIVITIES: &str = "a = .t~new~start('run', 'a')\nb = .t~new~start('run', 'b')\n\
     say a~result b~result\n::class t\n::method run\n  use arg tag\n  do i = 1 to 3\n  end\n  \
     return tag\n";

/// Main asleep while a started activity runs.
const SLEEPING: &str = "t = .t~new~start('nap')\ncall SysSleep 0.5\nsay t~result\n\
     ::class t\n::method nap\n  call SysSleep 1\n  return 'woke'\n";

/// Two activities in one guarded method of one object, the first asleep
/// holding its guard and the second queued for it.
const GUARDED: &str = "o = .g~new\na = o~start('hold', 'a')\nb = o~start('hold', 'b')\n\
     say a~result b~result\n::class g\n::method hold\n  use arg tag\n  call SysSleep 1\n  \
     return tag\n";

/// Each invariant fires on the state that breaks it, through the one
/// refusal the next boundary raises; the same runs unbroken end normally.
#[test]
fn each_invariant_is_refused_where_a_switch_finds_it_broken() {
    use crate::scheduler::Corruption;
    for (source, corruption, expected, answer) in [
        (
            THREE_ACTIVITIES,
            Corruption::ReadyTwice,
            "an activity ready twice",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::RunningReady,
            "an activity both running and ready",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::ReadyParked,
            "a ready activity holding a park reason",
            "a b\n",
        ),
        (
            SLEEPING,
            Corruption::NoWakeSource,
            "a parked activity with no wake source",
            "woke\n",
        ),
        (
            GUARDED,
            Corruption::GuardQueue,
            "a guard waiter with no wait recorded for its guard",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::Baton,
            "a switch on a thread not holding the baton",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::BogusReady,
            "a ready handle naming no idle activity",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::UnfiledHandle,
            "a handle neither free nor filed",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::FreeFiled,
            "a free handle naming an activity",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::RunningFree,
            "a running activity's handle free",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::RunningFiled,
            "a running activity filed as idle",
            "a b\n",
        ),
        (
            GUARDED,
            Corruption::GuardOwnerDead,
            "a guard held by no activity",
            "a b\n",
        ),
        (
            GUARDED,
            Corruption::GuardWaiterTwice,
            "a guard waiter queued twice or behind itself",
            "a b\n",
        ),
        (
            GUARDED,
            Corruption::GuardWaitUnqueued,
            "a guard wait missing from its guard's queue",
            "a b\n",
        ),
        (
            THREE_ACTIVITIES,
            Corruption::BatonReleased,
            "a switch on a thread not holding the baton",
            "a b\n",
        ),
    ] {
        let sound = corrupted(source, "sim:1", None);
        assert_eq!(sound.exit_code, 0, "{corruption:?}: {}", stderr(&sound));
        assert_eq!(stdout(&sound), answer, "{corruption:?}");
        let broken = corrupted(source, "sim:1", Some(corruption));
        assert_eq!(
            (broken.exit_code, stderr(&broken)),
            (120, format!("rexx-exec: the scheduler found {expected}\n")),
            "{corruption:?}"
        );
    }
}

/// `TIME('E')` after `SysSleep` is never 0, whatever the delay: on the
/// virtual clock a sleep shorter than a microsecond would otherwise end
/// inside the microsecond `TIME` read. The oracle read 0 in none of 2000
/// passes for a delay of 0 or 0.00000001.
#[test]
fn time_e_after_any_sleep_is_never_zero() {
    let program = "do d over 0, 0.0, 0.00000001, 0.000001\n\
                   call time 'R'\n\
                   call SysSleep d\n\
                   say d (time('E') > 0)\n\
                   end\n";
    for seed in 1..=20 {
        let outcome = run_in(program, Some(sim(&format!("sim:{seed},fifo"))));
        assert_eq!(outcome.exit_code, 0, "seed {seed}: {}", stderr(&outcome));
        assert_eq!(
            stdout(&outcome),
            "0 1\n0.0 1\n0.00000001 1\n0.000001 1\n",
            "seed {seed}"
        );
    }
}
