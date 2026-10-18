red = rexx built at e27a7f843 (/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.1-r1/rexx-head), green = rexx built at ade6fd223,
oracle = /home/moritz/dev/repos/ooRexx/build/bin/rexx. Each witness ran from a fresh empty directory.
reverse-chain-mutant: requires_dynamic_search_start.rex under a mutant that walks the package chain from the
outermost parent inward. It prints "parent directory" where the oracle prints "current directory".
unit-red.txt / unit-green.txt: cargo test --release -p rexx-exec --lib require before and after the require.rs change.
hid-requires/hidprobe.rex, run from a directory holding only sub/.hid.cls: the oracle prints "syntax 43.901",
and the binary at e27a7f843 prints "hid-cls".
