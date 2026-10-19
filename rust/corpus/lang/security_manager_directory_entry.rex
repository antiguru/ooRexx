/* A protected method stored with Directory~setMethod, or run with
   Object~run, runs without asking the security manager: an entry read and
   a run are not messages for a protected method. A plain send of it is. */
m = .Method~new('foo', 'return "m ran"')
m~setProtected
d = .Directory~new
d~setMethod('DIRM', m)
d~setMethod('UNKNOWN', m)
.local['D'] = d
.local['M'] = m
call show "send", "return .d~dirm"
call show "at", "return .d~at('DIRM')"
call show "brackets", "return .d['DIRM']"
call show "entry", "return .d~entry('DIRM')"
call show "unknown", "return .d~nosuch"
call show "unknown brackets", "return .d['NOSUCH']"
call show "run", "return .Holder~new~runit(.m)"
call show "object setMethod", "o = .Holder~new; o~hold(.m); return o~foo"
n = .Method~new('bar', 'if arg(1) == 2 then return "inner"; return "outer" self~bar(2)')
n~setProtected
.local['N'] = n
call show "run then send", "o = .Holder~new; o~hold(.n, 'BAR'); return o~runit(.n, 1)"
exit

show: procedure
  use arg label, code
  mgr = .Mgr~new
  r = .Routine~new("", code)
  r~setSecurityManager(mgr)
  say label r~call "events:" mgr~log~makeString(, " | ")
  return

::class Holder
::method hold
  use arg m, name = 'FOO'
  self~setMethod(name, m)
::method runit
  use arg m, a = ''
  return self~run(m, 'I', a)

::class Mgr
::method init
  expose log
  log = .Array~new
::method log
  expose log
  return log
::method unknown
  expose log
  use arg name, args
  info = args[1]
  log~append(name info['NAME'])
  if name \= "METHOD" then return 0
  info~result = "intercepted"
  return 1
