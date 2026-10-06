#!/bin/bash
# Runs single tests outside criterion 1's derived list alone, on the oracle
# and on REXX_RUN, from a copy laid out as group_runner.rs's fresh_copy does.
# Usage, from the repository root: bash alone.sh REXX_RUN SCRATCH
R=$1; S=$2; W=$(pwd)
one() {  # dir group test
  C=$S/copy
  rm -rf "$C"; mkdir -p "$C/ooRexx/$1"
  cp -r "$W/ootest/framework" "$C/"; cp -r "$W/ootest/ooRexx/$1/." "$C/ooRexx/$1/"
  cp "$W/ootest/testOORexx.rex" "$W/ootest/worker.rex" "$C/"
  python3 -c "
import sys
l=open(sys.argv[1]).read().split('\n'); h=[i for i,x in enumerate(l) if 'rxfuncquery(' in x]
d={j for i in h for j in (i-1,i,i+1)}; open(sys.argv[2],'w').write('\n'.join(x for i,x in enumerate(l) if i not in d))" "$W/ootest/ooTest.frm" "$C/ooTest.frm"
  cp "$W/extensions/rxregexp/rxregexp.cls" "$C/"
  cd "$C" || exit 1
  F=$C/ooRexx/$1/$2.testGroup
  o=$( (ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 60 /home/moritz/dev/repos/ooRexx/build/bin/rexx testOORexx.rex -f "$F" -U -V 2 -t "$3" 2>/dev/null) | grep -E '^(Assertions|Failures|Errors):' | tr -s ' ' | tr '\n' ' ')
  u=$(LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout 60 "$R" testOORexx.rex -f "$F" -U -V 2 -t "$3" 2>/dev/null | grep -E '^(Assertions|Failures|Errors):' | tr -s ' ' | tr '\n' ' ')
  echo "$2 $3 | oracle: $o| ours: $u"
  cd "$W"
}
for t in TEST_QUERYFILE_EXISTS_OPENED_01 TEST_QUERYFILE_EXISTS_OPENED_03 TEST_SEEK_CLOSEDFILE_2785896 TEST_SEEK_CLOSEDFILE_2787994 TEST_OPEN_WRITE_ONLY_3274050_C TEST_OPEN_WRITE_ONLY_3274050_D; do one base/bif STREAM $t; done
for t in TEST_RAISE_PROPAGATE TEST_RAISE_SYNTAX_EXIT_02 TEST_RAISE_SYNTAX_RETURN_02; do one base/keyword RAISE $t; done
for t in TEST_VALIDOPT_BIGCHAR_R TEST_VALIDOPT_LITTLECHAR_R; do one base/bif TIME $t; done
# A method does not start with a fresh elapsed clock (TIME's *_R tests in one run).
mkdir -p "$S/m"; cd "$S/m" || exit 1
printf "call time 'r'\nx = 0\ndo i = 1 to 300000; x = x + 1; end\n.o~new~m\n::class o\n::method m\n  y = time('R'); z = time('R'); say (y <= z)\n" > m.rex
echo "method clock probe | oracle: $( (ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx m.rex) ) | ours: $(timeout 20 "$R" m.rex)"
