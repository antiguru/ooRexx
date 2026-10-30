use super::*;

/// Step 1: where [`CALIBRATION_ENV`] names a file, runs each part
/// once under `fifo` (seed 0, no preemption but the floor) and
/// writes its contended steps, wall time and outcome; a whole part
/// whose run refuses gets a rest part leaving the refusing tests
/// out, as `whole_groups` derives it.
#[test]
fn calibration() {
    let Some(out) = std::env::var_os(CALIBRATION_ENV) else {
        eprintln!("sim_gate: no calibration without {CALIBRATION_ENV}");
        return;
    };
    let derived = derived_tests();
    let mut rows: Vec<Row> = Vec::new();
    let blank = |group: &str, part: &str, knobs: Option<&str>| Row {
        group: group.to_string(),
        part: part.to_string(),
        k: 0,
        release_seeds: 0,
        debug_seeds: 0,
        release_ms: 0,
        debug_ms: 0,
        knobs: knobs.map(str::to_string),
        left_out: Vec::new(),
    };
    for group in derived.keys() {
        rows.push(blank(group, "whole", None));
        rows.push(blank(group, "derived", None));
    }
    let list = rust_root().join("corpus/phase-6.txt");
    for line in std::fs::read_to_string(&list).expect("phase-6.txt").lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            rows.push(blank(&format!("corpus/{line}"), "program", None));
        }
    }
    for (program, knobs) in [
        ("m11_stale_sleeper.rex", Some("fail=wait:1")),
        ("n1_halt_ready_once.rex", Some("halt@30000")),
        ("timer_post_wakes_every_waiter.rex", None),
    ] {
        rows.push(blank(&format!("{PROGRAMS}{program}"), "program", knobs));
    }
    let lines = in_jobs(&rows, |at, row| {
        let mut row = row.clone();
        let mut out = String::new();
        let mut refusing: Vec<String> = Vec::new();
        loop {
            let dir = scratch("sim-calibration", at);
            let mode = row.mode(seed(&row.group, &row.part, 0), "fifo");
            let ran = run_here(&row, &derived, &dir, &mode, CALIBRATION_DEADLINE);
            remove(&dir);
            let mine = compared(&ran.run);
            let line = sim_line(&ran.run.stderr).unwrap_or_default();
            let contended = sim_field(&line, "contended").unwrap_or("0");
            let _ = writeln!(
                out,
                "{}\t{}\t{contended}\t{}\t{}\t{}\t{:?}\t{}",
                row.group,
                row.part,
                ran.took.as_millis(),
                row.knobs.as_deref().unwrap_or("-"),
                if refusing.is_empty() {
                    "-".to_string()
                } else {
                    refusing.join(" ")
                },
                ran.ended,
                outcome_key(&row, &mine)
            );
            if row.program() || row.part == "derived" {
                break;
            }
            let Some(line) = refusal(&mine) else { break };
            if line.ends_with("the run exceeded its deadline") {
                break;
            }
            let Some(test) = started(&mine).pop() else {
                break;
            };
            assert!(
                !refusing.contains(&test),
                "{}: {test} refused twice",
                row.group
            );
            refusing.push(test);
            row.part = "rest".to_string();
            row.left_out = refusing.clone();
        }
        out
    });
    std::fs::write(&out, lines.concat()).expect("cannot write the calibration");
}

