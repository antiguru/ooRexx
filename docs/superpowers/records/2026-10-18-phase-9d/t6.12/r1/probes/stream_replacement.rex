parse arg file
do v over .array~of(.nil, .object~new)
  mgr = .Mgr~new(v)
  r = .Routine~new("", "signal on syntax; call lineout arg(1), 'written'; return 'wrote'; syntax: return 'syntax' condition('o')~code condition('o')~message")
  r~setSecurityManager(mgr)
  say r~call(file)
  say "exists:" .File~new(file)~exists
end
::class Mgr
::method init
  expose v
  use arg v
::method unknown
  expose v
  use arg name, args
  if name \= 'STREAM' then return 0
  args[1]~stream = v
  return 1
