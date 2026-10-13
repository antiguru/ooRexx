/* SysGetMessage, SysGetMessageX, SysFormatMessage and SysGetErrorText: the
   interpreter's own messages by number and set, catalogs this program writes
   (found by a path, by NLSPATH and LANG, or not at all, and malformed ones),
   substitution with omitted, surplus, converted and raising values, the
   array size rule, strerror text, and the argument errors. */
here = directory()
d = 'messages.d'
call SysMkDir d
call SysMkDir d'/de_CH.UTF-8'
call SysMkDir d'/de'
call SysMkDir d'/CH'
call SysMkDir d'/UTF-8'
call SysMkDir d'/C'
call SysMkDir d'/dir.cat'
nl = '0a'x

say '-- strerror'
do n = -3 to 140
  say n SysGetErrorText(n)
end
call t "SysGetErrorText(2147483647)"
call t "SysGetErrorText(-2147483647)"
call t "SysGetErrorText(' 2 ')"
call t "SysGetErrorText(2.0)"
call t "SysGetErrorText(2147483648)"
call t "SysGetErrorText('x')"
call t "SysGetErrorText()"
call t "SysGetErrorText(1, 2)"

say '-- the interpreter''s messages'
found = 0
do n = 1 to 999
  m = SysGetMessage(n)
  if m == 'Error: Message not found' then iterate
  found += 1
  say n m
  if m \== SysGetMessageX(1, n, 'rexx.cat') then say n 'differs by SysGetMessageX'
end
say found 'found'
call t "SysGetMessage(5, 'rexx.cat')"
call t "SysGetMessage(5, 'rexx.cat' || '00'x || 'tail')"
call t "SysGetMessage(5, 'REXX.CAT')"
call t "SysGetMessage(5, '')"
call t "SysGetMessageX(2, 5)"
call t "SysGetMessageX(2, 5, 'rexx.cat')"
call t "SysGetMessageX(4294967297, 5)"
call t "SysGetMessage(4294967301)"
call t "SysGetMessage(999999999999999999)"
call t "SysGetMessage(678)"
call t "SysGetMessage(678, , 1)"
call t "SysGetMessage(678, , 1, 2, , 4)"
call t "SysGetMessage(678, , , , 3, 4, 5)"
call t "SysGetMessage(678, , 1, 2, 3, 4, 5, 6, 7, 8, 9)"
call t "SysGetMessage(678, , 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)"
call t "SysGetMessage(678, 'nowhere.cat', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)"
call t "SysGetMessage(678, , .array~of(1, 2), 'a' || '00'x || 'b')"
call t "SysGetMessage(0)"
call t "SysGetMessage(-1)"
call t "SysGetMessage(1.5)"
call t "SysGetMessage('x')"
call t "SysGetMessage(1000000000000000000)"
call t "SysGetMessage(.seven~new)"
call t "SysGetMessage(5, .arrayString~new)"
call t "SysGetMessage()"
call t "SysGetMessageX(1)"
call t "SysGetMessageX(0, 5)"

say '-- substitution'
call t "SysFormatMessage('&1', 'abc')"
call t "SysFormatMessage('&1&2', 'abc' || nl || 'def')"
call t "SysFormatMessage('&1 + &2 + &3', ('abc', , 'def'))"
call t "SysFormatMessage('&0 &1 && &', 'abc')"
call t "SysFormatMessage('&&1&', '&')"
call t "SysFormatMessage('&9 &8 &7 &6 &5 &4 &3 &2 &1 &0', .array~new(9)~fill(0)~allIndexes)"
call t "SysFormatMessage('a' || '00'x || '&1', 'q')"
call t "SysFormatMessage(.array~of('&1'), 'q')"
call t "SysFormatMessage('&1', 12)"
call t "SysFormatMessage('&1', .array~new(9))"
call t "SysFormatMessage('&1', .array~new(10))"
call t "SysFormatMessage('&1', .array~new(10)~~put('a', 1))"
a = .array~new; do 5; a~append('x'); end
call t "SysFormatMessage('&5', a)"
r = .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9, 10); r~delete(10)
call t "SysFormatMessage('&9', r)"
r = .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9, 10); r~remove(10)
call t "SysFormatMessage('&9', r)"
call t "SysFormatMessage('&1&2', .array~of(.array~of(1, 2), .object~new))"
call t "SysFormatMessage('&1', .array~of(.nil))"
call t "SysFormatMessage('&1', .array~of(.arrayString~new))"
call t "SysFormatMessage('&1', .array~of(.seven~new))"
call t "SysFormatMessage('&1', .array~of(.nilString~new))"
call t "SysFormatMessage('&1', .array~of(.noString~new))"
call t "SysFormatMessage('&2', .array~of(.loud~new('one'), .loud~new('two')))"
call t "SysFormatMessage('x', .array~of(.raiser~new('first'), .loud~new('after')))"
call t "SysFormatMessage('x', .array~of(.raiser~new('first'), .raiser~new('last')))"
call t "SysGetMessage(5, , .raiser~new('argument'))"
g = .array~new; g[1] = .grower~new(g); g[2] = 'b'
call t "SysFormatMessage('&1 &2', g)"
call t "SysFormatMessage('&1', .makesArray~new)"
call t "SysFormatMessage('&1', .array~new(2, 2))"
call t "SysFormatMessage('&1', .object~new)"
call t "SysFormatMessage('&1', .nil)"
call t "SysFormatMessage(.loud~new('message'), .array~of(1))"
call t "SysFormatMessage(.arrayString~new, .array~of(1))"
call t "SysFormatMessage(, 'q')"
call t "SysFormatMessage('&1')"
call t "SysFormatMessage()"
call t "SysFormatMessage('&1', 'q', 'r')"

say '-- catalogs'
entries = .array~of(.array~of(1, 1, 'one'), .array~of(1, 2, 'two &1 &2'), -
  .array~of(1, 3, ''), .array~of(2, 5, 'five'), .array~of(100, 1, 'far &2'))
call catalog d'/w.cat', 2, entries
cat = here'/'d'/w.cat'
call t "SysGetMessage(1, cat)"
call t "SysGetMessage(2, cat, 'a', 'b', 'c')"
call t "SysGetMessage(3, cat)"
call t "SysGetMessage(4, cat)"
call t "SysGetMessageX(2, 5, cat)"
call t "SysGetMessageX(100, 1, cat, 'x', 'y')"
call t "SysGetMessageX(3, 1, cat)"
call t "SysGetMessageX(2147483647, 1, cat)"
call t "SysGetMessageX(4294967296, 1, cat)"
call t "SysGetMessage(4294967297, cat)"
call t "SysGetMessage(4294967295, cat)"
call t "SysGetMessage(1, d'/w.cat')"
call t "SysGetMessage(1, './'d'/w.cat')"
call t "SysGetMessage(1, cat || '00'x || 'tail')"
call t "SysGetMessage(1, 'w.cat')"
call directory d
call t "SysGetMessage(1, './w.cat')"
call t "SysGetMessage(1, '../'d'/w.cat')"
call directory here

say '-- the search path'
call labelled d'/de_CH.UTF-8/w.cat', 'locale'
call labelled d'/de/w.cat', 'language'
call labelled d'/CH/w.cat', 'territory'
call labelled d'/UTF-8/w.cat', 'code set'
call labelled d'/C/w.cat', 'C'
call labelled 'w.cat', 'current directory'
call labelled '100%', 'percent'
call labelled d'/de/dir.cat', 'behind a directory'
oldPath = value('NLSPATH', , 'ENVIRONMENT')
oldLang = value('LANG', , 'ENVIRONMENT')
call search '', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%N', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%N', 'de_CH.UTF-8', 'none.cat'
call search d'/%N', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%L/%N', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%l/%N', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%t/%N', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%c/%N', 'de_CH.UTF-8', 'w.cat'
call search here'/'d'/%L/%N', '', 'w.cat'
call search here'/'d'/%t/%N', 'de', 'w.cat'
call search here'/'d'/%q/%N:'here'/'d'/%N', 'C', 'w.cat'
call search '/nowhere/%N::', 'C', 'w.cat'
call search here'/100%%', 'C', 'ignored'
call search here'/'d'/%N:'here'/'d'/de/%N', 'C', 'dir.cat'
call search here'/'d'/%N', 'C', ''
call value 'NLSPATH', oldPath, 'ENVIRONMENT'
call value 'LANG', oldLang, 'ENVIRONMENT'

say '-- files that are not catalogs'
call write d'/short.cat', 'de080496'x
call write d'/magic.cat', copies('00'x, 40)
call t "SysGetMessage(1, here'/'d'/short.cat')"
call t "SysGetMessage(1, here'/'d'/magic.cat')"
call t "SysGetMessage(1, here'/'d'/dir.cat')"
call t "SysGetMessage(1, here'/'d)"
whole = charin(d'/w.cat', 1, chars(d'/w.cat'))
call stream d'/w.cat', 'c', 'close'
call write d'/cut.cat', left(whole, 20)
call t "SysGetMessage(1, here'/'d'/cut.cat')"
call write d'/open.cat', left(whole, length(whole) - 1)
call t "SysGetMessage(1, here'/'d'/open.cat')"
call t "SysGetMessageX(100, 1, here'/'d'/open.cat')"
swapped = reverse(left(whole, 4)) || reverse(substr(whole, 5, 4)) || -
  reverse(substr(whole, 9, 4)) || substr(whole, 13)
call write d'/swapped.cat', swapped
call t "SysGetMessage(1, here'/'d'/swapped.cat')"

'rm -r' d 'w.cat 100%'
say SysFormatMessage('&1', .array~of(1, 2, 3, 4, 5, 6, 7, 8, 9, 10))
exit

t:
  parse arg expr
  signal on syntax name err
  interpret 'r =' expr
  say expr '->' '['r']'
  return
err:
  say expr '-> syntax' condition('o')~code ':' condition('o')~message
  return

/* SysGetMessage(1, name) with NLSPATH and LANG set as given. */
search: procedure expose here
  parse arg path, lang, name
  call value 'NLSPATH', path, 'ENVIRONMENT'
  call value 'LANG', lang, 'ENVIRONMENT'
  shown = changestr(here, path, '<here>')
  say '[' || shown || '] [' || lang || ']' name '->' SysGetMessage(1, name)
  return

write: procedure
  parse arg file, bytes
  call SysFileDelete file
  call charout file, bytes
  call stream file, 'c', 'close'
  return

le32: return reverse(x2c(d2x(arg(1), 8)))

/* A catalog whose message 1 of set 1 is `label`. */
labelled: procedure
  parse arg file, label
  call catalog file, 1, .array~of(.array~of(1, 1, label))
  return

/* A catalog as gencat writes one: (set, number, text) entries hashed into
   `size` buckets on (set + 1) * number, the table in host order, then
   swapped, then the messages. */
catalog: procedure
  use arg file, size, entries
  buckets = .array~new(size)
  do b = 1 to size; buckets[b] = .array~new; end
  strings = ''
  do e over entries
    b = ((e[1] + 1) * e[2]) // size + 1
    buckets[b]~append(.array~of(e[1] + 1, e[2], length(strings)))
    strings = strings || e[3] || '00'x
  end
  depth = 1
  do b = 1 to size; depth = max(depth, buckets[b]~items); end
  table = ''
  swapped = ''
  do level = 1 to depth
    do b = 1 to size
      entry = buckets[b][level]
      if .nil == entry then entry = .array~of(0, 0, 0)
      do w over entry
        table = table || le32(w)
        swapped = swapped || reverse(le32(w))
      end
    end
  end
  call write file, 'de080496'x || le32(size) || le32(depth) || table || swapped || strings
  return

::class seven
::method string; return 7
::class arrayString
::method string; return .array~of('s1', 's2')
::class nilString
::method string; return .nil
::class noString
::class loud
::method init; expose text; use arg text
::method string; expose text; say 'string of' text; return text
::class raiser
::method init; expose text; use arg text
::method string; expose text; raise syntax 93.900 array('raised' text)
::class grower
::method init; expose target; use arg target
::method string; expose target; target[2] = 'grown'; return 'g'
::class makesArray
::method makearray; return .array~of('m1', 'm2')
