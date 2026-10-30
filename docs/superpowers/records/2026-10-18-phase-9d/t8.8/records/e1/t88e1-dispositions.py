from pathlib import Path
import collections, hashlib, json, shutil, subprocess
root = Path('/home/moritz/dev/repos/ooRexx-rust-rewrite')
records = root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1'
fixed = root/'docs/superpowers/records/2026-10-18-phase-9d/t8.8/diagnostics/fixed-rechecks'
source = '1d8b03220'

# Re-read the durable descriptors rather than trusting the report's verdict words.
checks = []
names = ['stdin-original','stdin-one-line','stdin-multi-line','stdin-arrayin','stdin-fixed-chars','stdin-empty-eof','stdin-live-empty','trace-c','methods-original-counts','class-method-name-multisets','instance-method-name-multisets','instance-classobject-multiset','instance-existing-identity','package-copy-name','package-copy-lookups','rexx-routine-frame-executable']
for name in names:
    a,b = [fixed/f'{name}-{side}' for side in ['crate','oracle']]
    unordered = 'multiset' in name
    stdout = lambda p: collections.Counter((p/'stdout').read_bytes().splitlines()) if unordered else (p/'stdout').read_bytes()
    verdict = dict(name=name,stdout_equal=stdout(a)==stdout(b),stderr_equal=(a/'stderr').read_bytes()==(b/'stderr').read_bytes(),status_equal=(a/'status').read_bytes()==(b/'status').read_bytes(),stdout_multiset=unordered)
    assert all(verdict[key] for key in ['stdout_equal','stderr_equal','status_equal']), verdict
    checks.append(verdict)
for name, deviation in [('sysfork','42'),('named-event-create','43'),('named-event-open','43'),('named-mutex-create','43'),('named-mutex-open','43')]:
    directory = fixed/f'{name}-crate'
    assert (directory/'status').read_text().strip()=='120'
    assert not (directory/'stdout').read_bytes()
    assert f'Deviation {deviation}'.encode() in (directory/'stderr').read_bytes()
    checks.append(dict(name=name,status=120,deviation=deviation))
facts = json.loads((fixed/'static-facts.json').read_text())
for row in facts['production_refusal_rows']:
    assert hashlib.sha256(Path(row['source']).read_bytes()).hexdigest()==row['source_sha256']
(records/'consumed-fixed-evidence.json').write_text(json.dumps(checks,indent=2)+'\n')

inventory = root/'docs/superpowers/records/2026-10-18-phase-9d/queued-inventory.tsv'
rows = [line.split('\t') for line in inventory.read_text().splitlines()]
updates = {
    '2026-10-02-trace-entry-after-expose.md':'closed:fcb358fd7',
    '2026-10-02-traced-init-routed-sink-silent-exit.md':'closed:fcb358fd7',
    '2026-10-02-triggered-exit-trace-line-unrouted.md':'closed:fcb358fd7',
    '2026-10-18-max-integer-unrounded.md':'closed:009fd1978',
    '2026-10-18-fuzz-comparison-lostdigits.md':'closed:009fd1978',
}
queue_closures = {
    '2026-09-25-pos-window-doc-pointers.md':(source,'Restored POS-window and NUL-overrun mechanism from the pre-prune source, and replaced the stale expression visitor name with visit_expr.'),
    '2026-09-25-table-c-derivation-literals.md':(source,'Regenerated probe comments through the existing Table C derivation, verified the path-only diff, and passed the existing Table C checks.'),
    '2026-09-16-worktree-build-precondition.md':(source,'The completed fresh enumeration remains unchanged. The preflight refuses before Cargo with native, ooTest and reference paths, and its native recipe was built in an owned temporary out-of-source build.'),
    '2026-10-13-phase-9b-carry-in.md':(source,'The 9d minors are covered by the debug repeated-miss/write test, the already-correct objname mapping and the existing reused-class metaclass refusal witness. N1 remains Phase 9.1, and the whole-process budget belongs to the Phase 9 close owner. m2, samples and the SysSleep licence retain their existing dispositions.'),
    '2026-10-05-stdin-chars-after-drain.md':('probe agrees','Durable independent stdout, stderr and status rechecks cover piped EOF and a separately held-open empty pipe. Zero queries alone do not establish EOF.'),
    '2026-10-10-trace-c-command-echo.md':('probe agrees','The original ADDRESS COMMAND false shape agrees, including the clause echo and RC trace on raw stderr.'),
    '2026-10-13-named-semaphores-design.md':('probe agrees','Named event and mutex create/open all refuse at status 120 with Deviation 43. The unsupported ruling is unchanged.'),
    '2026-10-13-sysfork-design.md':('probe agrees','SysFork refuses at status 120 with Deviation 42. No oracle fork or implementation is claimed.'),
    '2026-10-13-yaml-pass-boundary-guard.md':('probe agrees','Both reductions agree, and TEST35_DIRECTIVES and TEST47_MULTILINEFLOWCOLLECTIONS pass in the full YAML group with both class copies. The private suite harness overlay changes only the selected crate-side yaml.cls source.'),
    '2026-10-18-class-methods-no-argument.md':('probe agrees','Complete inherited method-name multisets agree for no argument and .nil. Duplicate entries are preserved.'),
}
for filename,(revision, explanation) in queue_closures.items():
    path = root/'.superpowers/sdd/queued'/filename
    text = path.read_text()
    if filename=='2026-10-18-class-methods-no-argument.md':
        old = text
    else:
        old = text.split('\n',1)[1]
    scope = ' (9d minors)' if 'carry-in' in filename else ''
    path.write_text(f'**CLOSED 2026-10-30 by Phase 9d Task 8.8e1{scope} ({revision}).**\n\n'+old.lstrip('\n')+'\n\n## Task 8.8e1 evidence\n\n'+explanation+' See `.superpowers/sdd/2026-10-18-phase-9d/task-8.8e1-report.md` and `docs/superpowers/records/2026-10-18-phase-9d/t8.8/records/e1/`.\n')
    updates[filename]='closed:'+revision
for item in ['gate:phase-9a-gate.md:456','gate:phase-9a-gate.md:459','gate:phase-9a-gate.md:461','gate:phase-9a-gate.md:483']:
    updates[item]='closed:'+source
for item in ['gate:phase-9a-gate.md:474','gate:phase-9a-gate.md:488','gate:phase-9b1-gate.md:121']:
    updates[item]='closed:probe agrees'
updates['gate:phase-9a-gate.md:476']='phase:10'
for row in rows[1:]:
    if row[0] in updates:
        row[3]=row[5]=updates[row[0]]
inventory.write_text('\n'.join('\t'.join(row) for row in rows)+'\n')
fifo = root/'.superpowers/sdd/queued/2026-10-05-stream-fifo-terminal-seek.md'
fifo.write_text('**PARTIAL 2026-10-30 by Phase 9d Task 8.8e1.** FIFO ordinary transfer and bounded raw PTY read/write agree. Dynamic EINTR remains unprobed, and the oracle closing-LINEOUT error path consumes a byte after ESPIPE. Controller owns the remaining scope judgement.\n'+fifo.read_text().split('\n',1)[1]+'\n\n## Task 8.8e1 evidence\n\nThe bounded PTY control reads abc and writes xyz with matching descriptors and terminal bytes. Closing LINEOUT differs with or without a state query, and through the Stream method itself. The oracle syscall trace shows ESPIPE followed by a one-byte read that removes h from hello before close. This is an error-path side effect, not evidence that LINEOUT should consume FIFO data, and Deviation 46 is not broadened. See `task-8.8e1-report.md` for the retained EINTR diagnostic and controller handoff.\n')
closed = records/'closed.tsv'
closed.write_text('\n'.join(f'{key}\t{value}' for key,value in updates.items() if key.startswith('gate:') and value.startswith('closed:'))+'\n'+'gate:phase-9a-gate.md:476:rexx-package\tclosed:probe agrees\n'+'gate:phase-9a-gate.md:476:library-routine-executable\tphase:10:Rxapi\n')
discrepancies = []
for row in rows[1:]:
    path = root/'.superpowers/sdd/queued'/row[0]
    if path.is_file() and path.read_text().splitlines()[0].startswith('**CLOSED') and ('9d' in [row[3],row[5]]):
        discrepancies.append(row[0])
assert not discrepancies, discrepancies
(records/'inventory-file-state.json').write_text(json.dumps(dict(closed_file_open_9d_discrepancies=discrepancies,updates=updates,closure_record_provenance={'fcb358fd7':'Task 6.10 closure-record commit, not asserted as source implementation commit'}),indent=2)+'\n')
for temporary in ['/tmp/t88e1-check.py','/tmp/t88e1-additional.py','/tmp/t88e1-fifo-strace.py','/tmp/t88e1-yaml-harness.py','/tmp/t88e1-rederive.py','/tmp/t88e1-rederive.rs','/tmp/t88e1-dispositions.py']:
    shutil.copy2(temporary,records/Path(temporary).name)
cache = root/'rust/scripts/__pycache__'
if cache.exists():
    shutil.move(str(cache),'/tmp/t88e1-preflight-pycache')
print('Consumed independent fixed-recheck descriptors; updated assigned dispositions and found no CLOSED-file/open-9d mismatch.')
