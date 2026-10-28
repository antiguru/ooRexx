k = .k~new
say 'array hasitem'; x = .array~of(1)~hasItem(k)
say 'list hasitem'; x = .list~of(1)~hasItem(k)
say 'array index'; x = .array~of(1)~index(k)
say 'queue hasitem'; x = .queue~of(1)~hasItem(k)
say 'table hasitem'; x = .table~new~~put(1, 'a')~hasItem(k)
say 'set put'; s = .set~new; s~put(k)
say 'removeItem'; x = .array~of(1)~removeItem(k)
exit
::class k
::method '=='
  n = .context~stackFrames~items
  f = .context~stackFrames[2]
  say '  frames' n f~type f~name
  return 0
::method hashCode; return 'x'
