/* The control variable's own value lines on every re-tested pass. */
trace r
do ii = 1 to 3
  if ii = 2 then iterate
end
trace i
do jj = 2 to 1 by -1
  nop
end
do kk over 'ab'
  nop
end
trace off
say 'ended' ii jj kk
