/* An ITERATE naming an outer loop pauses once that loop has stepped, and
   the NOTREADY of that pause runs its handler there. */
call on notready name nr
trace ?a
do i = 1 to 2
  do j = 1 to 2
    if j = 1 then iterate i
  end
end
say 'after'
exit
nr: say 'nr' sigl; return
