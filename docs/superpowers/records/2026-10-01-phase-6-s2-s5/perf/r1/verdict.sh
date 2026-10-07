#!/bin/bash
# usage: verdict.sh TABLE : per program: name vs base %, vs s1 %, verdict (bar +1.0, fibfunc +2.58, rexxcps band 3388)
tail -n +2 $1 | awk -F'\t' '{bar=($1=="fibfunc")?2.58:1.0; band=($1=="rexxcps")?3388:0; d=($4-$2)/$2*100; over=(($4-$2-band)/$2*100>bar)?"OVER":"ok"; printf "%s\t%d\t%+.4f\t%+.4f\t%s\t%s\n",$1,$4,d,($4-$3)/$3*100,over,$9}'
