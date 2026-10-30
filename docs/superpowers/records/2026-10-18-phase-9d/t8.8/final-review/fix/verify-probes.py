import hashlib,json,pathlib
root=pathlib.Path('/home/moritz/dev/repos/ooRexx-rust-rewrite/docs/superpowers/records/2026-10-18-phase-9d/t8.8/final-review/fix')
rows=[]
for probe in sorted((root/'final').iterdir()):
 row={'probe':probe.name,'channels':{}}
 for channel in ['stdout','stderr','status']:
  data={side:(probe/side/channel).read_bytes() for side in ['oracle','baseline','crate']}
  assert data['oracle']==data['crate'], (probe.name,channel)
  if channel=='status':assert data['crate']==b'0\n',probe.name
  row['channels'][channel]={'oracle_matches_crate':True,'baseline_matches_crate':data['baseline']==data['crate'],'sha256':{side:hashlib.sha256(value).hexdigest() for side,value in data.items()}}
 rows.append(row)
assert len(rows)==20,len(rows)
(root/'paired-agreement.json').write_text(json.dumps(rows,indent=2)+'\n')
print('20 probes, 60 exact oracle/crate channel agreements; every final status 0')
