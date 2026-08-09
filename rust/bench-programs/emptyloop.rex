/* The clause-dispatch floor: a loop whose body does nothing, so the
   measurement is the per-clause cost and almost nothing else. */
n = 25000000
do i = 1 to n
  nop
end
say 'done'
