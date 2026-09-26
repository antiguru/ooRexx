#!/bin/bash
# build.sh OUTDIR : the Task 5 re-review's extension, librr and librr2 into OUTDIR,
# and librr linked with a static libgcc into OUTDIR/static
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O/static"
D=$(dirname $(readlink -f $0))
I="-I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix"
gcc -c -fPIC -O1 -funwind-tables -DFN=c_call_uw $D/rrc.c -o $O/rrc_uw.o
gcc -c -fPIC -O1 -fno-unwind-tables -fno-asynchronous-unwind-tables -DFN=c_call_nouw $D/rrc.c -o $O/rrc_nouw.o
g++ -shared -fPIC -O1 -static-libstdc++ $I $D/rr.cpp $O/rrc_uw.o $O/rrc_nouw.o -o $O/librr.so
g++ -shared -fPIC -O1 -static-libstdc++ -DSECOND $I $D/rr.cpp $O/rrc_uw.o $O/rrc_nouw.o -o $O/librr2.so
g++ -shared -fPIC -O1 -static-libstdc++ -static-libgcc $I $D/rr.cpp $O/rrc_uw.o $O/rrc_nouw.o -o $O/static/librr.so
for f in librr.so librr2.so static/librr.so; do
  echo "$f: $(readelf -d $O/$f | grep NEEDED | sed 's/.*\[\(.*\)\]/\1/' | tr '\n' ' ')"
  nm -D --undefined-only $O/$f | grep -i rexx || echo "$f: no undefined Rexx symbol"
done
