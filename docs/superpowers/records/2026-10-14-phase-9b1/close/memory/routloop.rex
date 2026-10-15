parse arg n
do i = 1 to n
  r = .Routine~new("r", "return" i)
end
say done
