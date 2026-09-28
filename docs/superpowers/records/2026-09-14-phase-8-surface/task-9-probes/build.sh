#!/bin/bash
# build.sh OUTDIR : build libouter9.so and libouter9b.so against the frozen headers
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O"
D=$(dirname $(readlink -f $0))
for n in outer9 outer9b; do
  g++ -shared -fPIC -O1 -static-libstdc++ -I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix $D/$n.cpp -o $O/lib$n.so
  readelf -d $O/lib$n.so | grep NEEDED || echo "no NEEDED entry"
  nm -D --undefined-only $O/lib$n.so | grep -i rexx || echo "no undefined Rexx symbol"
done
