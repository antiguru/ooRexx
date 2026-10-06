import re,sys
src=open('/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs').read()
cur=None
for m in re.finditer(r'static\s+([A-Z_]+)\s*:|\(\s*"(Message|EventSemaphore|MutexSemaphore)",\s*"([^"]+)"',src):
    if m.group(1): cur=m.group(1); continue
    line=src.count('\n',0,m.start())+1
    print(f"dispatch.rs:{line}\t{cur}\t{m.group(2)}\t{m.group(3)}")
