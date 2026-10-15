parse arg n
o = .object~new
do i = 1 to n
  m = .Message~new(o, "STRING")
  m~send
end
say done
