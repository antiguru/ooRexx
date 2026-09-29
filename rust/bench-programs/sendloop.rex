/* An empty ::METHOD sent n times. */
n = 5000000
o = .sink~new
do i = 1 to n
    o~ping
end
say 'done'

::class sink
::method ping
