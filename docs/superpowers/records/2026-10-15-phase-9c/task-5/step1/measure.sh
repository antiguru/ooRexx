#!/bin/bash
# measure.sh BIN DIR : run every Step 1 shape with BIN from the fixture directory DIR.
B=$1
cd "$2" || exit 1
R=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/9c-t5/tools/run1.sh
r() { bash "$R" "$@"; }
FF=$(printf '\xff')

r no-arguments "$B"
r v "$B" -v
r V "$B" -V
r v-then-unknown "$B" -v -?
r unknown-then-v "$B" -? -v
r o-then-v "$B" -o 'digits 12' -v
r v-after-program "$B" t.rex -v
r v-after-e "$B" -e 'say 1' -v
r e-code "$B" -e 'say 1'
r E-code "$B" -E 'say 2'
r e-no-code "$B" -e
r e-empty "$B" -e ''
r e-args "$B" -e "say arg() '['arg(1)']'" a 'b c' d
r e-no-args "$B" -e "say arg() '['arg(1)']'"
r e-syntax-error "$B" -e 'say ('
r e-exit-7 "$B" -e 'exit 7'
r e-return-300 "$B" -e 'return 300'
r e-raise-42 "$B" -e 'say 1/0'
r e-parse-source "$B" -e 'parse source s; say s'
r e-syscargs "$B" -e 'say .syscargs; say .local~syscargs'
r e-twice "$B" -e 'say 1' -e 'say 2'
r e-then-o "$B" -e 'say digits()' -o 'digits 12'
r o-then-e "$B" -o 'digits 12' -e 'say digits()'
r o-digits "$B" -o 'digits 12' opts.rex
r od-digits "$B" -od 'digits 12' opts.rex
r O-digits "$B" -O 'digits 12' opts.rex
r OD-digits "$B" -OD 'digits 12' opts.rex
r o-colon "$B" -o '::options digits 12' opts.rex
r o-single-colon "$B" -o ':options digits 12' opts.rex
r o-second-colon "$B" -o 'x:options digits 12' opts.rex
r o-leading-blank-colon "$B" -o ' ::options digits 12' opts.rex
r o-two-blanks-colon "$B" -o '  ::options digits 12' opts.rex
r o-trace "$B" -o 'trace all' opts.rex
r o-own-options "$B" -o 'digits 12' ownopts.rex
r od-own-options "$B" -od 'digits 12' ownopts.rex
r o-requires "$B" -o 'digits 12' req.rex
r od-requires "$B" -od 'digits 12' req.rex
r o-last "$B" -o
r od-last "$B" -od
r o-malformed "$B" -o 'bogus 9' opts.rex
r o-malformed-value "$B" -o 'digits x' opts.rex
r o-empty "$B" -o '' opts.rex
r o-twice "$B" -o 'digits 12' -o 'form engineering' opts.rex
r o-syscargs "$B" -o 'digits 12' showargs.rex a b
r od-syscargs "$B" -od 'digits 12' showargs.rex a b
r unknown-x "$B" -x showargs.rex a b
r unknown-x-y "$B" -x -y showargs.rex a b
r unknown-x-no-args "$B" -x showargs.rex
r unknown-only "$B" -x
r bare-dash "$B" - showargs.rex a
r bare-dash-only "$B" -
r plain-args "$B" showargs.rex a 'b c' d
r plain-no-args "$B" showargs.rex
r plain-empty-arg "$B" showargs.rex ''
r dash-args-after-program "$B" showargs.rex -o 'my output file'
r find-rex-extension "$B" t
r find-upper "$B" UP
r find-sub "$B" sub/t
r find-dot-sub "$B" ./sub/t
r find-noext "$B" noext
r find-directory "$B" dironly
r find-directory-ext "$B" dironly.rex
REXX_PATH="$PWD/rp" r find-rexx-path "$B" viarp
PATH="$PWD/pp:$PATH" r find-path "$B" viapp
r not-found "$B" nonexist
r not-found-ext "$B" nonexist.rex
r not-found-dir "$B" sub/nonexist
r parse-source "$B" src.rex
r parse-source-dot "$B" ./sub/../src.rex
r error-42 "$B" err42.rex
r error-35 "$B" err35.rex
for n in abc neg3 e300 f15 big nbig null none bare r255 r256 sp e1 obj; do
  r "return-$n" "$B" "ret_$n.rex"
done
r name-blank "$B" 'my prog.rex'
r name-blank-noext "$B" 'my prog'
r name-ff "$B" "p${FF}.rex"
r name-ff-noext "$B" "p${FF}"
r name-ff-file-noext "$B" "q${FF}"
r arg-ff "$B" bytes.rex "a${FF}b"
r o-empty-then-colon-name "$B" -o '' ':x'
r o-blank-then-colon-name "$B" -o '  ' ':x'
LONG="digits 12$(printf '%600s' '') form engineering"
r o-long "$B" -o "$LONG" opts.rex
r o-then-o-last "$B" -o 'digits 12' -o
r o-suffix-x "$B" -ox 'digits 12' req.rex
r o-suffix-D "$B" -oD 'digits 12' req.rex
r O-suffix-d "$B" -Od 'digits 12' req.rex
r e-suffix "$B" -ev 'say 3'
r v-suffix "$B" -vx
r double-dash "$B" -- showargs.rex a
r unknown-then-e "$B" -x -e 'say 4'
r e-syscargs-items "$B" -e 'say .syscargs~items'
for spec in "abc:return 'abc'" "neg3:return -3" "big16:return 70000" "f15:return 1.5" "big:return 2147483648" "none:nop" "r256:return 256" "sp:return ' 4 '" "e1:return 1e1"; do
  name=${spec%%:*}; body=${spec#*:}
  r "e-return-$name" "$B" -e "$body"
done
r e-name-in-error "$B" -e 'say 1' x 'y z'
r e-with-o-own "$B" -o 'digits 12' -e '::options digits 5'
