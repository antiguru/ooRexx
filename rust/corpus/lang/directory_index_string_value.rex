/* A string-keyed collection takes every index as a string argument, so an
 * index with no string value is 88.909 on each index-taking method, not a
 * lookup that answers.  An object whose makeString answers is keyed by that
 * answer.  Table, whose index is compared by equality, accepts the same
 * objects as indexes.
 */

signal on syntax name trapped
n = 0
d = .Directory~new
t = .StringTable~new

next:
n = n + 1
select
  when n = 1 then say n d~at(.nil)
  when n = 2 then say n d[.Directory~new]
  when n = 3 then say n d~put('x', .nil)
  when n = 4 then do; d[.nil] = 'x'; say n; end
  when n = 5 then say n d~hasIndex(.nil)
  when n = 6 then say n d~remove(.nil)
  when n = 7 then say n d~entry(.nil)
  when n = 8 then say n d~hasEntry(.nil)
  when n = 9 then do; d~setEntry(.nil, 'x'); say n; end
  when n = 10 then say n d~removeEntry(.nil)
  when n = 11 then do; d~setMethod(.nil, 'return 1'); say n; end
  when n = 12 then do; d~unsetMethod(.nil); say n; end
  when n = 13 then say n t~at(.nil)
  when n = 14 then say n t~hasIndex(.nil)
  when n = 15 then say n t~remove(.nil)
  when n = 16 then do; t~put('x', .nil); say n; end
  otherwise signal done
end
signal next

trapped:
say n 'raised' condition('C') rc'.'condition('E')~substr(pos('.', condition('E')) + 1)
signal on syntax name trapped
signal next

done:
signal off syntax
k = .Keyed~new
d[k] = 'stored under KEYED'
say 'keyed   ' d['KEYED'] d~hasIndex(k) d~allIndexes~makeString('L', ',')
tab = .Table~new
tab[.nil] = 'nil as a table index'
say 'table   ' tab[.nil] tab~hasIndex(.nil)

::class Keyed
::method makeString
  return 'KEYED'
