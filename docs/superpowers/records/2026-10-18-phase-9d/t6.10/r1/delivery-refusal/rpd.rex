::class refusing public
::method init
  expose n
  n = 0
::method lineout
  expose n
  use arg traced
  n = n + 1
  line = traced~makeString
  call lineout 'log.txt', n word(line, 1) word(line, 2) word(line, 3)
  if n = 5 then do
    s = .Message~new(.Stem, .array~of('NEW', .Class))~send
    x = s~items
  end
  return 0
