.traceOutput~destination(.counting~new)
a = .tracer~new~start('run')
b = .tracer~new~start('run')
ra = a~result
rb = b~result
.traceOutput~destination
say 'lines' .counting~count
::class tracer
::method run
  trace r
  do i = 1 to 30
    x = 1
  end
  trace off
  return 0
::class counting
::method init class
  expose count
  count = 0
::method bump class
  expose count
  count += 1
::method count class
  expose count
  return count
::method lineout
  .counting~bump
  return 0
