/* LINES Count after CHARIN on a named stream */
f = arg(1)
say 'A' lines(f, 'C') lines(f)
say '[' || charin(f, , 2) || ']'
say 'B' lines(f, 'C') lines(f)
say '[' || linein(f) || ']'
say 'C' lines(f, 'C') lines(f)
say '[' || linein(f) || ']'
say 'D' lines(f, 'C') lines(f)
say '[' || charin(f, , 7) || ']'
say 'E' lines(f, 'C') lines(f)
say '[' || linein(f) || ']'
say 'F' lines(f, 'C') lines(f)
say '[' || linein(f) || ']'
say 'G' lines(f, 'C') lines(f)
