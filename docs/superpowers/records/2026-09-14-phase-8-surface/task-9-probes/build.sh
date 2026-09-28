#!/bin/bash
# build.sh OUTDIR : build libouter9.so from outer9.cpp against the frozen headers
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O"
D=$(dirname $(readlink -f $0))
g++ -shared -fPIC -O1 -static-libstdc++ -I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix $D/outer9.cpp -o $O/libouter9.so
readelf -d $O/libouter9.so | grep NEEDED || echo "no NEEDED entry"
nm -D --undefined-only $O/libouter9.so | grep -i rexx || echo "no undefined Rexx symbol"
