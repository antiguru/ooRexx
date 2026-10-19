.traceOutput~destination(.refusing~new)
m = .k~new~start('m')
r = m~result
say 'after'
::class k
::method m
  trace l
  return 1
::class refusing
::method init
  expose n
  n = 0
::method lineout
  expose n
  n = n + 1
  if n = 2 then do
    s = .Message~new(.Stem, .array~of('NEW', .Class))~send
    x = s~items
  end
  return 0
