code = "say 'interpreted'"
interpret code
v = 0
interpret "v = 6 * 7"
say v
do i = 1 to 3
  interpret "say 'loop" i "'"
end
