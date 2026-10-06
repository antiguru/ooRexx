/* Message round trip: a message started on a new activity and its result
   awaited, n times. */
n = 80000
o = .echo~new
total = 0
do i = 1 to n
    total = total + o~start('back', 1)~result
end
say total

::class echo
::method back
    use arg i
    return i
