say .OrderedCollection~method('SORTWITH')~source~items
say .OrderedCollection~method('SORT')~source~items
say .OrderedCollection~method('STABLESORT')~source~items
say .OrderedCollection~method('STABLESORTWITH')~source~items
say .Collection~method('HASITEM')~source~items
say 1
say .array~method('SORT')~source~items
do c over .array~of(.OrderedCollection, .Collection, .MapCollection, .SetCollection, .Orderable, .Comparable, .TraceObject, .File, .Monitor, .Alarm, .Ticker, .DateTime, .TimeSpan, .StreamSupplier, .InputStream, .Stream, .RexxQueue, .CaselessComparator, .Properties)
  n = 0; t = 0
  s = c~methods(.nil)
  do while s~available
    t += 1; if s~item~source~items > 0 then n += 1
    s~next
  end
  say c~id t n
end
