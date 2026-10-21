/* the same sequence on a named stream */
f = arg(1)
say '[' || charin(f, 2, 0) || ']' chars(f)
say '[' || linein(f) || ']' chars(f)
say '[' || linein(f, 3) || ']' chars(f)
say '[' || linein(f) || ']' chars(f)
say '[' || linein(f, 2) || ']' chars(f)
say '[' || charin(f, , 2) || ']' chars(f)
say '[' || linein(f, 4) || ']' chars(f)
