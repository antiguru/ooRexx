/* kind n: build an acyclic chain of the kind with leaf 'leaf' (or 7), time positions */
parse arg kind n
signal on syntax
select
  when kind = 'ref' then do       /* v1 = >v2, v2 = >v3, ..., vn = 'leaf' */
    do i = 1 to n - 1
      interpret 'v'i' = >v'i + 1
    end
    interpret 'v'n' = 7'
    first = v1
  end
  when kind = 'plain' then do     /* plain .stem~new instances, each the previous one's default */
    first = .stem~new; t = first
    do i = 2 to n; u = .stem~new; t[] = u; t = u; end
    t[] = 7
  end
  when kind = 'sub' then do
    first = .m~new; t = first
    do i = 2 to n; u = .m~new; t[] = u; t = u; end
    t[] = 7
  end
  when kind = 'mix' then do       /* instance -> reference to a stem -> instance ... */
    first = .m~new; t = first
    do i = 1 to n % 2
      interpret 't[] = >s'i'.'
      u = .m~new
      interpret 's'i'. = u'
      t = u
    end
    t[] = 7
  end
  when kind = 'stemref' then do   /* s1.'s default a reference to s2. ... */
    do i = 1 to n - 1
      interpret 's'i'. = >s'i + 1'.'
    end
    interpret 's'n'. = 7'
    first = >s1.
  end
end
call time 'R'; x = length(first); say 'length()' x time('E')
call time 'R'; x = first || 'x'; say 'concat' x time('E')
call time 'R'; x = first + 1; say 'arith' x time('E')
call time 'R'; x = (first = 7); say 'compare' x time('E')
call time 'R'; x = first~string; say '~string' x time('E')
call time 'R'; x = first~length; say '~length' x time('E')
call time 'R'; x = first~request('STRING'); say '~request' x time('E')
call time 'R'; x = first~makestring; say '~makestring' x time('E')
call time 'R'; z.1 = 'a'; x = z.first; say 'tail' x time('E')
call time 'R'; x = datatype(first); say 'datatype' x time('E')
call time 'R'; x = .array~of(first)~toString; say 'array~tostring' x time('E')
call time 'R'; c. = first; x = c.zz; y = length(c.zz); say 'compound' y time('E')
call time 'R'; x = .context~variables~items; say 'ctxvars' time('E')
call time 'R'; parse var first p; say 'parsevar' p time('E')
call time 'R'; x = value('first'); x = length(x); say 'value()' x time('E')
call time 'R'; if first = 7 then nop; say 'if' time('E')
call time 'R'; trace r; x = first; trace o; say 'trace' time('E')
say 'end'
exit
syntax: say 'SYNTAX' condition('O')~code condition('O')~position; exit
::class m subclass stem
