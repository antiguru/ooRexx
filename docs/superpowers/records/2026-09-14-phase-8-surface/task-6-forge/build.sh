#!/bin/bash
# build.sh OUTDIR : build libcmd.so from cmd.cpp against the frozen headers
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O"
D=$(dirname $(readlink -f $0))
g++ -shared -fPIC -O1 -static-libstdc++ -I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix $D/cmd.cpp -o $O/libcmd.so
readelf -d $O/libcmd.so | grep NEEDED
nm -D --undefined-only $O/libcmd.so | grep -i rexx || echo "no undefined Rexx symbol"
