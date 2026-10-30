use super::*;

#[test]
fn timeout_context_names_the_probe_phase_budget_and_load() {
    let context = timeout_context(
        Path::new("probe.rex"),
        "child wait",
        Duration::from_secs(10),
        Duration::from_secs(12),
        Ok(b"4.00 3.00 2.00 1/100 42\n"),
    );
    assert!(context.contains("probe.rex"), "{context}");
    assert!(context.contains("child wait"), "{context}");
    assert!(context.contains("10s"), "{context}");
    assert!(context.contains("12s"), "{context}");
    assert!(
        context.contains("loadavg=4.00 3.00 2.00 1/100 42"),
        "{context}"
    );
}

#[test]
fn unavailable_load_keeps_the_timeout_diagnostic_useful() {
    let error = std::io::Error::from(std::io::ErrorKind::NotFound);
    let context = timeout_context(
        Path::new("drain.rex"),
        "output drain",
        Duration::from_secs(1),
        Duration::from_secs(2),
        Err(&error),
    );
    assert!(context.contains("output drain"), "{context}");
    assert!(context.contains("loadavg=unavailable"), "{context}");
    assert!(context.contains(&error.to_string()), "{context}");
    assert_eq!(classify_termination(None, true), Termination::TimedOut);
    assert_eq!(
        classify_termination(Some(17), true),
        Termination::Exited(17)
    );
}
