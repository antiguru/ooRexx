#!/bin/bash
# build.sh OUTDIR : the loader probes' libraries (liblr, liblr2, liblt, liblt2) against the frozen headers
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O"
D=$(dirname $(readlink -f $0))
I="-I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix"
for n in lr lt; do
  g++ -shared -fPIC -O1 -static-libstdc++ $I $D/$n.cpp -o $O/lib$n.so
  g++ -shared -fPIC -O1 -static-libstdc++ -DSECOND $I $D/$n.cpp -o $O/lib${n}2.so
done
readelf -d $O/liblr.so $O/liblr2.so $O/liblt.so $O/liblt2.so | grep NEEDED | sort -u
for f in liblr liblr2 liblt liblt2; do
  nm -D --undefined-only $O/$f.so | grep -i rexx || echo "$f: no undefined Rexx symbol"
done
