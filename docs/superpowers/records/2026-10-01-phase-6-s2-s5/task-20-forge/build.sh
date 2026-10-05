#!/bin/bash
# build.sh OUTDIR : build libfinish.so and liblent.so against the frozen headers
O=${1:?usage: build.sh OUTDIR}; mkdir -p "$O"
D=$(dirname $(readlink -f $0))
for lib in finish lent; do
  g++ -shared -fPIC -O1 -static-libstdc++ -I/home/moritz/dev/repos/ooRexx/api -I/home/moritz/dev/repos/ooRexx/api/platform/unix $D/$lib.cpp -o $O/lib$lib.so || exit 1
done
