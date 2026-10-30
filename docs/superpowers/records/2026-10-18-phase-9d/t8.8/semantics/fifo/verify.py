from pathlib import Path
import json,hashlib
root=Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/semantics/fifo')
rows=[]
for name in ['crate-r4','oracle-r4','partial-crate-r1','partial-oracle-r1']:
 d=root/name;desc=json.loads((d/'descriptor.json').read_text());tid=desc['blocked_before_signal']['tid'];fd=desc['blocked_before_signal']['fd'];trace=(d/f'trace.{tid}').read_text();fifo=desc['blocked_before_signal']['fd_link']
 assert desc['status']==0 and desc['elapsed']<15 and 'failure' not in desc
 assert desc['observation']=='exited after signal without writer unblock'
 assert not any('unblock' in e['event'] for e in desc['events'])
 assert desc['ready_hex']=='52454144590a'
 assert '= -1 EINTR' in trace
 after=trace.split('= -1 EINTR',1)[1]
 assert not any(f'read({fd}<{fifo}>' in line for line in after.splitlines())
 out=(d/'stdout').read_text();err=(d/'stderr').read_bytes()
 assert 'read []\n' in out and 'ERROR:4 Interrupted system call' in out and err==b''
 assert (d/'events').read_bytes()==b'NOTREADY ERROR:4 Interrupted system call\nHALT ERROR:4 Interrupted system call\n'
 if name.startswith('partial'):
  assert any(f'read({fd}<{fifo}>' in line and '"P"' in line and '= 1' in line for line in trace.splitlines())
 for p in d.glob('trace.*'):
  assert not Path('/proc',p.suffix[1:]).exists(),p
 rows.append({'name':name,'status':desc['status'],'elapsed':desc['elapsed'],'stdout':out,'checks':'EINTR, no retry/unblock, empty result, NOTREADY before HALT, owned tasks reaped'})
(root/'verification.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows))
