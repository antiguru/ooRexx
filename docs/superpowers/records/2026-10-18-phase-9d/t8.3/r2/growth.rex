parse arg shape n
.local~rr = .routines
do i = 1 to n
  select
    when shape = 'interp' then do
      .routines['Q'] = .routine~new('q', 'return arg(1)')
      interpret 'x = q(i)'
    end
    when shape = 'nested' then do
      .routines['G'] = .routine~new('g', 'return arg(1)')
      r = .routine~new('r', 'return g(arg(1))')
      x = r~call(i)
    end
    when shape = 'site' then do
      .routines['P'] = .routine~new('p', 'use arg v; .rr~remove("P"); return v')
      x = viap(i)
    end
  end
end
say shape x
exit
viap: return p(arg(1))
::routine dummy
