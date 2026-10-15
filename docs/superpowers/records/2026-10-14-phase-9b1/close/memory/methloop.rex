parse arg n
do i = 1 to n
  m = .Method~new("m", "return" i)
end
say done
