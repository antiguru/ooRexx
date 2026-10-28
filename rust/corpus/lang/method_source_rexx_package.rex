/* The REXX package's methods have no source; a program's own do. */
say .OrderedCollection~method('SORTWITH')~source~items
say .OrderedCollection~method('SORT')~source~items
say .Collection~method('HASITEM')~source~items
say .array~method('SORT')~source~items
say .k~method('M')~source~items
say .k~instanceMethod('R')~source~items
say .method~new('x', 'return 1')~source~items
do c over .array~of(.OrderedCollection, .Collection, .Orderable, .Comparable, .DateTime, .TimeSpan, .Stream, .File, .RexxQueue, .Properties)
  n = 0
  s = c~methods(.nil)
  do while s~available
    if s~item~source~items > 0 then n += 1
    s~next
  end
  say c~id n
end
exit
::class k
::method m
  return 1
::method r class
  nop
