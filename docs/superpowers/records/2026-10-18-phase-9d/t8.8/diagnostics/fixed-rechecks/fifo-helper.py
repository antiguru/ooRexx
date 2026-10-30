from pathlib import Path
import sys,time
p=Path(sys.argv[1])
if sys.argv[2]=="write":
 time.sleep(0.3)
 with p.open("wb",buffering=0) as f: f.write(b"abc")
else:
 with p.open("rb",buffering=0) as f: data=f.read()
 Path(sys.argv[3]).write_bytes(data)
