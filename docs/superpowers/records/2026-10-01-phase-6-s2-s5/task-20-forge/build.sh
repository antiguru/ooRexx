#!/bin/bash
# build.sh OUTDIR : build libfinish.so from finish.cpp against the frozen headers
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O"
D=$(dirname $(readlink -f $0))
g++ -shared -fPIC -O1 -static-libstdc++ -I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix $D/finish.cpp -o $O/libfinish.so
