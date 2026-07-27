/* String dimension: SUBSTR/POS/CHANGESTR/concatenation. */
n = 3000000
s = "the quick brown fox jumps over the lazy dog"
total = 0
do i = 1 to n
    p = pos("fox", s)
    piece = substr(s, p, 3)
    changed = changestr("fox", s, "cat")
    joined = piece || changed
    total = total + length(joined)
end
say total
