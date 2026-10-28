t0 = time('E')
address system '(sleep 1; echo late) &' with output stem s.
say 'stem' s.0 'waited' (time('E') - t0 > 0.8)
t0 = time('R')
address system '(sleep 1; echo late2 >&2) &' with error stem e.
say 'err' e.0 'waited' (time('E') > 0.8)
t0 = time('R')
address system 'echo now; (sleep 1; echo late3) &'
say 'plain waited' (time('E') > 0.8)
call syssleep 1.5
say 'end'
