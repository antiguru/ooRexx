/* the file grows after it was read to the end */
do while lines() > 0
  l = linein()
end
say 'A' chars() lines() lines(, 'C')
address system 'printf "more\nlines\n" >> ' || arg(1)
say 'B' chars() lines() lines(, 'C')
say '[' || linein() || ']'
say 'C' chars() lines() lines(, 'C')
