#!/bin/bash
# build.sh OUTDIR : build libreach.so from reach.cpp against the frozen headers
O=$1; mkdir -p $O
D=$(dirname $(readlink -f $0))
g++ -shared -fPIC -O1 -I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix $D/reach.cpp -o $O/libreach.so
readelf -d $O/libreach.so | grep NEEDED
nm -D --undefined-only $O/libreach.so | grep -i rexx
