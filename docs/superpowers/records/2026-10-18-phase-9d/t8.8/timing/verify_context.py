import json, pathlib, re
records = pathlib.Path(__file__).parent
log = (records / 'final-affected-targets.txt').read_text()
contexts = []
for line in log.splitlines():
    if line.startswith('oracle timeout: '):
        match = re.fullmatch(r'oracle timeout: probe=(.*?) phase=(child wait|output drain) configured=(.*?) elapsed=(.*?) loadavg=(.*)', line)
        assert match, line
        contexts.append(dict(zip(['probe', 'phase', 'configured', 'elapsed', 'loadavg'], match.groups())))
for name, phase in [('hang.rex', 'child wait'), ('orphan.rex', 'output drain')]:
    rows = [row for row in contexts if row['probe'].endswith('/' + name)]
    assert len(rows) == 1 and rows[0]['phase'] == phase and rows[0]['configured'] == '10s', rows
    assert rows[0]['loadavg']
source = pathlib.Path('crates/rexx-exec/tests/oracle_deadline.rs').read_text()
assert source.count('oracle.invocations(),\n        1,') == 2
for name in ['a_program_that_never_finishes_reddens_at_the_deadline_instead_of_hanging', 'a_background_process_holding_the_pipe_open_does_not_hang_the_run']:
    assert f'test {name} ... ok' in log
(records / 'deadline-context-assertion.json').write_text(json.dumps(dict(status='passed', contexts=contexts, single_invocation_controls='Both original controls asserted oracle.invocations() == 1 and passed in final-affected-targets.txt; no retry was added.'), indent=2) + '\n')
print('Deadline path/phase/budget/load associations and single invocation controls: passed')
