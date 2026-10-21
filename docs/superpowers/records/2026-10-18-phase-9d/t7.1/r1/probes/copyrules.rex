do o over .array~of(.Array, .zz, .context, .RexxInfo)
  call plain o
  call override o
end
exit 0
plain: procedure
  use arg o
  signal on syntax name a
  c = o~copy
  say 'plain copied'
  return
a: say 'plain' condition('O')~code condition('O')~message
  return
override: procedure
  use arg o
  signal on syntax name d
  c = .Message~new(o, .array~of('COPY', .Object))~send
  say 'override copied' c~class~id
  return
d: say 'override' condition('O')~code condition('O')~message
  return
::class zz
