/* An Alarm whose target is a Message sends that message when it fires. */
t = .target~new
m = .message~new(t, 'ring', 'I', 'x')
a = .alarm~new(0.5, m)
say 'main' a~triggered
::class target
::method ring
  say 'ring' arg() arg(1) (.context~thread \= 1)
