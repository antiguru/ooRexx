n = 0
do while lines() > 0 & n < 5
  n = n + 1
  say n '[' || linein() || ']'
end
say 'lines end' n lines()
