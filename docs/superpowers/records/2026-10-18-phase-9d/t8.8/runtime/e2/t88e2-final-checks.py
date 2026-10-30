import subprocess, sys
root='/home/moritz/dev/repos/ooRexx-rust-rewrite/rust'
checks=[
 ('focused-final','180',['cargo','test','-j4','-p','rexx-exec','--lib','spawn_failure','--','--nocapture']),
 ('stdin-metadata','180',['cargo','test','-j4','-p','rexx-exec','--test','stdin_contention','--test','owners','--test','closed_phases','--test','phase_ten_reasons','--test','representation_refusals']),
 ('sourceline','90',['cargo','test','-j4','-p','rexx-parse','--test','sourceline_oracle']),
 ('release-build','180',['cargo','build','-j4','--release','-p','rexx-exec','--bin','rexx']),
 ('fmt-complete','60',['cargo','fmt','--all','--check']),
]
for name,bound,cmd in checks:
 status=subprocess.call([sys.executable,'/tmp/t88e2-check.py',name,bound,root,*cmd])
 if status: sys.exit(status)
