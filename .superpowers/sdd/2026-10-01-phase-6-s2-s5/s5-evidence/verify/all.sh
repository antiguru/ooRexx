#!/bin/bash
for p in u1 u2 u3 u4 u5 t3 t7; do
  for side in oracle ours; do
    bash /tmp/claude-1000/p6-s5-verify/run.sh $side $p 30
  done
done
