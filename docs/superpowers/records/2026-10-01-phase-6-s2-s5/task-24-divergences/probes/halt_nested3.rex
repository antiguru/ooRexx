g = .gate~new
a = .t~new~start('a')
b = .t~new~start('b', g)
do while \g~started; end
a~halt
g~sent = 1
say 'halt sent'
::class gate
::attribute started unguarded
::attribute sent unguarded
::method init
  expose started sent
  started = 0
  sent = 0
::class t
::method a unguarded
  signal on halt name h
  interpret 'do forever; end'
h:
  say 'A halted'
::method b unguarded
  use arg g
  interpret 'g~started = 1; do until g~sent; end; do i = 1 to 1000000; end'
  say 'B done'
