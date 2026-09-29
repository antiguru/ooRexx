/* A ::METHOD sent in a loop whose body does nothing: the cost of a send and
   its return, without dispatch.rex's EXPOSE and arithmetic. */
n = 5000000
o = .sink~new
do i = 1 to n
    o~ping
end
say 'done'

::class sink
::method ping
