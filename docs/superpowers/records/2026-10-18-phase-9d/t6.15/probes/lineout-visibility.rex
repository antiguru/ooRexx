/* When a LINEOUT to a named stream becomes visible to another reader:
   a second stream object on the same file, and another process. */
call run 'implicit', ''
call run 'open both', 'open both'
call run 'open nobuffer', 'open both nobuffer'
call run 'flush', 'open both', 'flush'
exit
run:
  parse arg name, opts, extra
  f = 'v' || name~changestr(' ', '_') || '.txt'
  if opts \== '' then call stream f, 'c', opts
  call lineout f, 'first'
  call charout f, 'second'
  if extra == 'flush' then call stream f, 'c', 'flush'
  other = .stream~new(f)
  seen = other~charin(1, other~chars)
  other~close
  address system 'wc -c <' f with output stem w.
  say name ': other object' length(seen) '| other process' strip(w.1) '| size' stream(f, 'c', 'query size')
  call stream f, 'c', 'close'
  say name ': after close size' stream(f, 'c', 'query size')
  return
