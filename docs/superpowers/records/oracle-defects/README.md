# Oracle defects

Known defects in the C++ ooRexx 5.3 interpreter (crashes, wrong results, inconsistencies,
concurrency, documentation mismatches), one object per defect in `defects.json`. Each entry carries
`added`, the date it entered this list. Published as https://claude.ai/artifact/JbRX9HW5j3tvt7gbksb3Gw;
rebuild with `python3 build.py OUT.html` and republish OUT.html to that URL.

Fields: id, added, title, category, area, severity, repro, oracle, expected, cause, source, status,
crate. Sources: rust/corpus/oracle-crashes.txt, the numbered Deviations in
docs/superpowers/plans/phase-4-exclusions.txt that are oracle defects, queued items of kind
`oracle bug`, and filed SourceForge tickets.
