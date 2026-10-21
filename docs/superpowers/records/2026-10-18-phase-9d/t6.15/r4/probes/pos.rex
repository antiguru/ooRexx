/* positioning on a standard input: the shared read position */
say 'A' chars() lines() lines(, 'C')
say '[' || linein() || ']'
say 'B' chars() lines() lines(, 'C')
say '[' || charin(, 3) || ']'
say 'C' chars() lines() lines(, 'C')
say '[' || charin(, 1, 3) || ']'
say 'D' chars() lines() lines(, 'C')
say '[' || linein() || ']'
say 'E' chars() lines() lines(, 'C')
say '[' || charin(, , 3) || ']'
say 'F' chars() lines() lines(, 'C')
say '[' || linein() || ']'
say 'G' chars() lines() lines(, 'C')
say '[' || linein(, 1) || ']'
say 'H' chars() lines() lines(, 'C')
say '[' || linein(, 3) || ']'
say 'I' chars() lines() lines(, 'C')
say '[' || .stdin~charin(2, 2) || ']'
say 'J' chars() lines() lines(, 'C')
say '[' || .stdin~linein(4) || ']'
say 'K' chars() lines() lines(, 'C')
say '[' || charin(, 22) || ']'
say 'L' chars() lines() lines(, 'C')
say '[' || linein() || ']'
say 'M' chars() lines() lines(, 'C')
