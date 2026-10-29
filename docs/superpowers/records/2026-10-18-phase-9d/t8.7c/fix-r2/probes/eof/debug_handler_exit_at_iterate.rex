/* A CALL ON handler that EXITs from the pause after an ITERATE returns from
   the method while its loop is still open; the caller's loop calls it again. */
o = .c~new
do j = 1 to 2
  say 'got' o~m(j)
end
say 'done' j
exit
::class c
::method m
  use arg j
  call on notready name nr
  trace ?a
  do i = 1 to 3
    iterate
  end
  return 'normal'
nr: if sigl = 15 then exit 'early' j i; return
