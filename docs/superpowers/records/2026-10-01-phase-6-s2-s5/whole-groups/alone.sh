#!/bin/bash
# Runs single tests alone, on the oracle and on REXX_RUN, from a copy laid out
# as group_runner.rs's fresh_copy does, each side with its own `rexx` on PATH
# and standard input empty. Prints each side's summary and our refusal.
# Usage, from the repository root: bash alone.sh REXX_RUN SCRATCH
R=$1; S=$2; W=$(pwd)
O=/home/moritz/dev/repos/ooRexx/build
mkdir -p "$S/bin"; ln -sf "$R" "$S/bin/rexx"
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
  o=$( (ulimit -v 1048576; PATH=$O/bin:$PATH LD_LIBRARY_PATH=$O/lib timeout -s KILL 60 $O/bin/rexx testOORexx.rex -f "$F" -U -V 2 -t "$3" 2>/dev/null </dev/null) | grep -E '^(Assertions|Failures|Errors):' | tr -s ' ' | tr '\n' ' ')
  u=$(PATH=$S/bin:$PATH LD_LIBRARY_PATH=$O/lib timeout 60 "$R" testOORexx.rex -f "$F" -U -V 2 -t "$3" 2>"$S/err.txt" </dev/null | grep -E '^(Assertions|Failures|Errors):' | tr -s ' ' | tr '\n' ' ')
  echo "$2 $3 | oracle: $o| ours: $u$(grep -m1 '^rexx-exec: ' "$S/err.txt")"
  cd "$W"
}
for t in TEST_QUERYFILE_EXISTS_OPENED_01 TEST_QUERYFILE_EXISTS_OPENED_03 TEST_SEEK_CLOSEDFILE_2785896 TEST_SEEK_CLOSEDFILE_2787994 TEST_OPEN_WRITE_ONLY_3274050_C TEST_OPEN_WRITE_ONLY_3274050_D; do one base/bif STREAM $t; done
for t in TEST_RAISE_PROPAGATE TEST_RAISE_SYNTAX_EXIT_02 TEST_RAISE_SYNTAX_RETURN_02; do one base/keyword RAISE $t; done
for t in TEST_VALIDOPT_BIGCHAR_R TEST_VALIDOPT_LITTLECHAR_R; do one base/bif TIME $t; done
for t in TEST_ACTIVATE TEST_METHODS TEST_CLASS_DEFINE; do one base/class Class $t; done
for t in TEST_INSTANCEMETHOD TEST_RUN TEST_SETMETHOD TEST_SETMETHOD_SCOPE TEST_RUN_ARRAY_ARGUMENT; do one base/class Object $t; done
for t in TESTRS01 TESTCONDITION01; do one base/class RexxContext $t; done
for t in TEST_EXPRESSION_NOVALUE_ERROR TEST_BAD_NEGATIVE; do one base/directives CONSTANT $t; done
one base/directives ATTRIBUTE TESTABSTRACTTWICE
one base/directives METHOD TESTABSTRACTEXTERNAL
one base/keyword CALL TEST_INVALID
one base/keyword GUARD TEST_INVALID_OPTION_ONOFF
for t in 'TEST_TRACE_?' 'TEST_TRACE_?A' 'TEST_TRACE_?I' 'TEST_TRACE_?R' 'TEST_TRACE_?_OPTION' TEST_TRACE_DROP TEST_TRACE_EXIT TEST_TRACE_EXPOSE TEST_TRACE_IGNORED TEST_TRACE_NUMERIC_DEBUG TEST_TRACE_OTHER_ENTRYPOINT TEST_TRACE_PROCEDURE TEST_TRACE_LABEL_WITH_FORWARD; do one base/keyword TRACE "$t"; done
for t in TEST_CALLER_STACK_FRAME TEST_CALLER_STACK_FRAME_REPLY_START; do one base/keyword TRACE_TraceObject $t; done
one base/class DateTime TEST_BRUTE_FORCE
one base/class Message TEST_REPLYWITH_NOT_ARRAY
one base/class MethodArgs TEST_REQUEST_STRING_CLASS
one base/class Method TESTDIRECTIVES
one doc/rexxref/chapter5 Section1 TEST_OBJECT_OBJECTNAMEEQUALS
# A method does not start with a fresh elapsed clock: whole-groups/clock/.
