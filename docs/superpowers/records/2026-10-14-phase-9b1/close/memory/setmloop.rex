parse arg n
d = .directory~new
do i = 1 to n
  d~setMethod("f", "return" i)
end
say done d~f
