/* NOSTRING additional object, one shape per run */
parse arg w
signal on nostring
d = .directory~new
if w = 1 then do; r = >d; say 'a' r; end
if w = 2 then do; s. = d; say 'b' s.; end
if w = 3 then do; s. = d; q = >s.; say 'c' q; end
if w = 4 then do; s. = d; q = >s.; u = >q; say 'd' u; end
if w = 5 then do; t = .mystem~new; t[] = d; say 'e' t; end
if w = 6 then do; t = .mystem~new; t[] = d; s. = t; say 'f' s.; end
if w = 7 then do; t = .mystem~new; t[] = d; s. = t; q = >s.; say 'g' q; end
if w = 8 then do; s. = d; say 'h' length(s.); end
if w = 9 then do; s. = d; q = >s.; say 'i' (1 + q); end
say 'end' w
exit
nostring: say 'NS' w condition('D') condition('O')~class~id condition('A')~class~id; exit
::class mystem subclass stem
