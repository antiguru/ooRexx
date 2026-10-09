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
    ] {
        let config = SimConfig::parse(text).expect("a sim spec");
        assert_eq!(config.to_string(), text);
    }
    assert_eq!(
        SimConfig::parse("sim:7").map(|c| c.to_string()),
        Ok("sim:7,fifo".into())
    );
    assert_eq!(
        SimConfig::parse("sim:7,block=0").map(|c| c.to_string()),
        Ok("sim:7,fifo,block=0".into())
    );
    assert_eq!(
        SimConfig::parse("sim:7,block=1e30"),
        Err("`block=1e30`: block is a number of seconds from 0 to 86400".into())
    );
    for wrong in [
        "sim:",
        "sim:x",
        "sim:1,gc=2",
        "sim:1,halt@0",
        "sim:1,fail=wait:",
        "sim:1,block=-1",
        "sim:1,block=x",
        "sim:1,block=inf",
        "sim:1,block=NaN",
        "sim:1,block=1e30",
        "sim:1,pre:1",
        "simx",
    ] {
        assert!(SimConfig::parse(wrong).is_err(), "{wrong} parsed");
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
