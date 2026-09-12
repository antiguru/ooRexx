/* A read goes to .INPUT as a message: LINEIN and CHARIN with no arguments,
   LINES with 'NORMAL', and PULL as a LINEIN of its own. */
s = .src~new
zz = .input~destination(s)
say 'linein=[' || linein() || ']'
say 'lines=[' || lines() || ']'
say 'charin=[' || charin() || ']'
pull zv
say 'pull=[' || zv || ']'
zz = .input~destination(.stdin)
say 'seen' s~seen~items
do msg over s~seen
  say msg
end

::class src
::method init
  expose seen
  seen = .array~new
::method seen
  expose seen
  return seen
::method linein
  expose seen
  seen~append('LINEIN n=' arg())
  return 'a line'
::method lines
  expose seen
  use arg opt
  seen~append('LINES n=' arg() 'opt=' opt)
  return 7
::method charin
  expose seen
  seen~append('CHARIN n=' arg())
  return 'c'
