-- Main ends inside the pinned yield of a started activity spinning inside
-- INTERPRET; the program ends once that activity does, with main's status.
g = .flag~new
g~start('spin')
do while g~started == 0
end
say 'main sets'
g~done = 1
say 'main ends'
exit 3
::class flag
::attribute started unguarded
::attribute done unguarded
::method init
  expose started done
  started = 0
  done = 0
::method spin unguarded
  self~started = 1
  interpret "do while \self~done; end"
  say 'spin ended'
