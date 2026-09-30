/* An INIT that raises, reached through ~new from a send. */
say 'before'
x = .k~new(0)
say 'after' x
::class k
::method init
  use arg n
  say 'init' n
  return 1 / n
