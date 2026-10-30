use super::*;

/// The rows marked [`UNSTABLE`] whose two oracle runs answer the same thing.
pub fn stable_rows_marked_unstable(layout: &Layout) -> Vec<String> {
    let receivers = layout.receivers();
    let dir = probe_dir(layout.probe_prefix);
    let path = dir.join("probe.rex");
    if layout.fixture {
        fs::write(dir.join(FIXTURE), "return 1\n").expect("the fixture is writable");
    }
    let oracle = oracle::oracle_root();
    let mut stable = Vec::new();
    for (class, method, arm, list) in layout.arguments() {
        if !list.starts_with(UNSTABLE) {
            continue;
        }
        let receiver = receivers
            .get(&(class.clone(), arm.clone()))
            .unwrap_or_else(|| panic!("{} has no setup for {class} ({arm})", layout.receivers));
        fs::write(&path, program(receiver, &method, NONE, true)).expect("the probe is writable");
        let first = run(&mut oracle_command(&oracle, &path, &dir));
        let second = run(&mut oracle_command(&oracle, &path, &dir));
        if first == second {
            stable.push(format!(
                "{class}~{method} ({arm}) is marked {UNSTABLE} and the oracle answers {:?} on \
                 two runs of its own, so the value is comparable -- drop the marker",
                last_line(&first.1)
            ));
        }
    }
    stable
}


#[cfg(test)]
mod tests;
