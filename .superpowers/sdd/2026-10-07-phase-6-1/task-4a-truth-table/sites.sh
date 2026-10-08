#!/bin/bash
# usage: sites.sh REV
# Every line of rexx-exec/src outside tests that names a truth primitive or a
# text compare against a logical value, for reading off which sites judge
# truth and through what.
cd /home/moritz/dev/repos/ooRexx-rust-rewrite || exit 9
git grep -n -E 'logical_value\(|is_true_object|LOGICAL_TRUE|LOGICAL_FALSE|unwrap_or\(false\)|== b"1"|== b"0"|b"1" =>|b"0" =>|truth\(|truth_without_conversion\(' "$1" -- rust/crates/rexx-exec/src ':!*tests.rs' ':!*/tests/*'
