#!/bin/bash
# fixture.sh DIR : populate the empty directory DIR with the Step 1 programs.
set -e
d=$1
cd "$d"
mkdir sub rp pp dironly.rex
printf "say 'hi from t'\n" > t.rex
printf "say 'hi from sub/t'\n" > sub/t.rex
printf "say 'hi from rp'\nparse source s; say s\n" > rp/viarp.rex
printf "say 'hi from pp'\nparse source s; say s\n" > pp/viapp.rex
printf "say 'upper T.REX'\n" > UP.REX
printf "say 'noext file'\nparse source s; say s\n" > noext
printf "parse source s; say s\n" > src.rex
printf "say 'arg()='arg() 'arg(1)=['arg(1)']'\ns = .syscargs\nsay 'items='s~items 'size='s~size 'last='s~last\ndo i = 1 to s~last\n  if s~hasIndex(i) then say i'=['s[i]']'\n  else say i' empty'\nend\nsay '['s~toString(, ', ')']'\n" > showargs.rex
printf "say 'digits='digits() 'form='form()\nsay 'trace='trace()\n" > opts.rex
printf "say 'digits='digits()\n::options digits 5\n" > ownopts.rex
printf "say 'main digits='digits()\ncall libr\n::requires 'lib.rex'\n" > req.rex
printf "::routine libr public\nsay 'lib digits='digits()\n" > lib.rex
printf "say 1/0\n" > err42.rex
printf "say 'x' +\n" > err35.rex
for spec in "abc:return 'abc'" "neg3:return -3" "e300:exit 300" "f15:return 1.5" "big:return 2147483648" "nbig:return -2147483649" "null:return ''" "none:nop" "bare:exit" "r255:return 255" "r256:return 256" "sp:return ' 4 '" "e1:return 1e1" "obj:return .array~new" ; do
  name=${spec%%:*}; body=${spec#*:}
  printf "%s\n" "$body" > "ret_$name.rex"
done
printf "parse source s; say s\nsay 1/0\n" > 'my prog.rex'
printf "parse source s; say s\nsay 1/0\n" > "$(printf 'p\xff.rex')"
printf "parse source s; say s\nsay 1/0\n" > "$(printf 'q\xff')"
printf "say c2x(arg(1))\nsay c2x(.syscargs[1])\n" > bytes.rex
