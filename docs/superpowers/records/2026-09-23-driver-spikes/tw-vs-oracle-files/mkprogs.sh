#!/bin/bash
# Writes the micro-programs: one counted loop around one construct, plus controls.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
P=$S/progs
mkdir -p "$P"
pre="n = 200000; y = 5; x = 0; a = 'abc'; b = 'abd'; s = 'one two three'"
fill="do j = 1 to n; a.j = j; end"
mk() {
    {
        echo "$2"
        [ -n "$4" ] && echo "$4"
        echo "do i = 1 to n"
        [ -n "$3" ] && echo "  $3"
        echo "end"
        echo "say x"
        echo "exit"
        echo "r: return"
    } > "$P/$1.rex"
}
mk ctrl "$pre" ""
mk ctrl2 "${pre/n = 200000/n = 400000}" ""
mk ctrlfill "$pre" "" "$fill"
mk nop "$pre" "nop"
mk assign_var "$pre" "x = y"
mk assign_lit "$pre" "x = 'abc'"
mk incr "$pre" "x = x + 1"
mk concat "$pre" "x = a || b"
mk if_eq "$pre" "if a = b then nop"
mk stem_store "$pre" "a.i = x"
mk stem_read "$pre" "x = a.i" "$fill"
mk bif_length "$pre" "x = length(y)"
mk call_r "$pre" "call r"
mk parse_var "$pre" "parse var s a b c"
mk mul "$pre" "x = y * 1.5"
