/* Note: uses explicit || rather than abuttal. A symbol abutting a preceding
   string literal can be read as its hex/binary suffix -- say a"|"b parses
   "|"b as a binary literal and fails with error 15. */
parse value "alpha beta gamma" with p1 p2 p3
say p1 || "/" || p2 || "/" || p3
parse value "2026-07-27" with yy "-" mm "-" dd
say yy || ":" || mm || ":" || dd
parse value "one two three four" with first rest
say first || "[" || rest || "]"
parse value "abcdef" with 3 mid 5 tail
say mid || "," || tail
parse upper value "MiXeD" with u
say u
parse value "a,b,c" with f1 "," f2 "," f3
say f1 f2 f3
