# criterion 6 over the in-process ooTest harnesses, at 2a9bbbe12, from rust/: each harness's
# test run alone, every in-process run of ours logged by watchdog::log_sharing:
# REXX_SHARING_LOG=keyword.tsv memcap 8G cargo test --release -p rexx-exec --features sharing --test keyword_assertions -- --exact keyword_assertions_differential
# REXX_SHARING_LOG=bif.tsv memcap 8G cargo test --release -p rexx-exec --features sharing --test bif_assertions -- --exact bif_assertions_differential
# REXX_SHARING_LOG=expressions.tsv memcap 8G cargo test --release -p rexx-exec --features sharing --test assertions -- --exact assertions_differential
# REXX_CORPUS_GATE=1 REXX_SHARING_LOG=api.tsv memcap 8G cargo test --release -p rexx-exec --features sharing --test api_group_tests -- --exact every_test_of_the_phase_8_groups_passes_and_matches_the_oracle_but_the_recorded
# then, per log: awk -F'\t' '{bo+=$2; bs+=$3; po+=$4; ps+=$5} END{print NR, bo, bs, po, ps}'

| harness | runs | bootstrap objects | bootstrap shared | program objects | program shared |
|---|---|---|---|---|---|
| keyword | 896 | 243712 | 0 | 15675 | 0 |
| bif | 10184 | 2770048 | 0 | 12609 | 0 |
| expressions | 4259 | 1158448 | 0 | 10858 | 0 |
| api (each test of API/oo CONVERSION, FUNCTION, METHOD, but those reaching rxapi) | 577 | 156944 | 0 | 1222310 | 0 |
