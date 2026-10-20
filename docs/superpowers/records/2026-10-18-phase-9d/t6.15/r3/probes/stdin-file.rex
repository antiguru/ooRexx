/* CHARS and LINES on a standard input redirected from a regular file. */
say 'C' chars() .stdin~chars lines() .stdin~lines
say '[' || linein() || ']'
say 'D' chars() .stdin~chars lines() .stdin~lines
say '[' || charin(, 3) || ']'
say 'E' chars() .stdin~lines
n = 0
do while lines() > 0 & n < 9
  n = n + 1
  say n '[' || linein() || ']'
end
say 'end' n chars() lines()
