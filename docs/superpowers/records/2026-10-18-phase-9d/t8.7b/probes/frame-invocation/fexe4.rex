interpret 'do i = 1 to 2; say .context~stackframes[1]~invocation .context~invocation; end'
signal on syntax
interpret 'a = .context~stackframes[1]~invocation; say "a" a; interpret "say .context~stackframes[1]~invocation .context~stackframes[2]~invocation"; raise syntax 98.900 array("x")'
exit
syntax:
  c = condition('o')
  do f over c~stackframes
    say 'cond' f~type f~line (f~invocation == .nil)
  end
  say 'again' .context~stackframes[1]~invocation .context~invocation
