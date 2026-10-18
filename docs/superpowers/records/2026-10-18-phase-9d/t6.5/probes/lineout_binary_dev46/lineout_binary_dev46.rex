/* LINEOUT TEST_BINARY's sequence: five padded records, then a position-only
   LINEOUT, a write, and a close by LINEOUT. Deviation 46. */
name = 'b.bin'
say stream(name, 'c', 'open write shared replace binary reclength 5')
do n = 5 to 1 by -1
  say lineout(name, n~copies(n), n)
end
call stream name, 'c', 'flush'
say '[' || charin(name, 1, 25) || ']'
say lineout(name, , 3)
say lineout(name, 'three')
say lineout(name)
say '[' || charin(name, 1, 30) || ']'
