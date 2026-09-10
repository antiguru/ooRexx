/* Values that arrive as text and are then used as numbers -- the operation
   rexxcps performs 5.58 million times and no bench program performs at all.
   Each iteration builds fresh strings, so the handles differ and a
   handle-keyed cache cannot answer from the previous iteration. */
total = 0
do i = 1 to 200000
  s = substr('1234567890', 3, 4)
  t = substr('9876543210', 2, 3)
  if s > t then total = total + 1
  total = total + s + t
  if s = 3456 then total = total + 1
end
say total
