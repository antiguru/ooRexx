#!/bin/bash
# usage: run.sh OUTDIR NAME=BINARY... -- callgrind Ir per line program, per size, per binary
out=$1; shift
mkdir -p $out
here=$(dirname $(readlink -f $0))
for n in 2500 5000; do
  f=$out/lines$n.txt
  python3 -I -c "import sys; open(sys.argv[1],'w').write(''.join('line %06d of the file\n' % i for i in range(1, int(sys.argv[2])+1)))" $f $n
done
printf 'program\tlines'; for b in "$@"; do printf '\t%s' ${b%%=*}; done; printf '\n'
for p in p1 p2 p4 p6; do
  for n in 2500 5000; do
    printf '%s\t%s' $p $n
    for b in "$@"; do
      name=${b%%=*}; bin=${b#*=}
      cg=$out/$p.$n.$name.cg
      ( cd $out && memcap 4G timeout 900 valgrind --tool=callgrind --callgrind-out-file=$cg $bin $here/$p.rex "lines$n.txt" $n >$cg.out 2>$cg.err )
      ir=$(grep -m1 '^summary:' $cg | awk '{print $2}')
      printf '\t%s' $ir
    done
    printf '\n'
  done
done
