import hashlib, json, pathlib
root = pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root / 'docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix-r1'
fixtures = root / 'rust/crates/rexx-exec/tests/semantic_boundaries_stdout'
results = []
for probe in sorted((records / 'probes').glob('*.rex')):
    for channel in ['stdout', 'stderr', 'status']:
        values = {side: (records / 'final' / probe.stem / side / channel).read_bytes()
                  for side in ['oracle', 'baseline', 'crate']}
        assert values['oracle'] == values['crate'], (probe.stem, channel)
        expected = b'0\n' if channel == 'status' else (fixtures / (probe.stem + '.' + channel)).read_bytes()
        assert values['oracle'] == expected, (probe.stem, channel, 'fixture')
        results.append(dict(probe=probe.stem, channel=channel, oracle_crate_equal=True,
                            baseline_equal=values['baseline'] == values['oracle'],
                            sha256={side: hashlib.sha256(value).hexdigest() for side, value in values.items()}))
(records / 'paired-agreement.json').write_text(json.dumps(results, indent=2) + '\n')
print(f'{len(results)} exact oracle/crate channel agreements; fixtures independently match oracle')
