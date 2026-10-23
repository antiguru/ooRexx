parse arg shape n
.local~rr = .routines
do i = 1 to n
  select
    when shape = 'site' then do
      .routines['P'] = .routine~new('p', 'use arg v; .rr~remove("P"); return v')
      x = viap(i)
    end
    when shape = 'raise' then do
      .routines['E'] = .routine~new('e', 'raise syntax 40.1 array("E")')
      x = trye(i)
    end
    when shape = 'nested' then do
      .routines['G'] = .routine~new('g', 'return arg(1)')
      r = .routine~new('r', 'return g(arg(1))')
      x = r~call(i)
    end
    when shape = 'interp' then do
      .routines['Q'] = .routine~new('q', 'return arg(1)')
      interpret 'x = q(i)'
    end
    when shape = 'cycle' then do
      .routines['A'] = .routine~new('a', 'if arg(1) > 0 then return b(arg(1) - 1); return 0')
      .routines['B'] = .routine~new('b', 'return a(arg(1))')
      x = .routines['A']~call(1)
    end
    when shape = 'native' then do
      .routines['S'] = .routine~new('s', 'return 0')
      .array~of(1, 2)~sortWith(.cmp~new)
      x = i
    end
  end
end
say shape x
exit
viap: return p(arg(1))
trye:
  signal on syntax name caught
  return e(arg(1))
  caught:
  return 0
::routine dummy
::class cmp
::method compare
  return s()
