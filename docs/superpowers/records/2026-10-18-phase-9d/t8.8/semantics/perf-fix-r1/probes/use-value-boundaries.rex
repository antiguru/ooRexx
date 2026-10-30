/* Supplied immediates, heap strings and objects survive argument binding. */
call bind 12, 'xy', copies('heap', 8), .box~new('supplied')
call bind , , ,
exit
bind:
  use arg number = 34, text = 'ab', heap = (copies('default', 4)), object = (.box~new('default'))
  say number text length(heap) object~value
  return
::class box
::method init
  expose value
  use arg value
::attribute value
