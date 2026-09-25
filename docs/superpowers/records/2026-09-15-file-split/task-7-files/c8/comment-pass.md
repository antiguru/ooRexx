# c8 comment pass: `builtin/string.rs`'s `mod scan_tests`

Source: `c8-comments.txt` (comments7.py) and `c8-positional.txt`.

* (b) `corpus/phase-4c.txt:192`: see c7's pass; `scan_tests` holds
  `find_byte_agrees_with_position`, which is not the test that sentence
  means, so this commit changes nothing about it.
* The moved doc, "[`find_byte`]'s word step against the byte-at-a-time
  answer it stands in for": `find_byte` is imported by `use
  super::find_byte;` in the same module `builtin::string::scan_tests`, so
  the link resolves as before.
* No comment anywhere names `scan_tests` or its test; no positional word
  in the moved text.
