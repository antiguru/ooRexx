/* The REQUIRES checkpoint's NAME is the file to load: a replacement that
   cannot be read, .nil among them, is 3.1 naming it after both events;
   removing the entry forbids the load with 43.901 naming the original. */
call lineout 'smreq.cls', "say 'smreq loaded'"
call lineout 'smreq.cls'
call try "object", .object~new
exit

try: procedure
  use arg label, replacement
  mgr = .Mgr~new(replacement)
  r = .Routine~new("", "signal on syntax; return .Package~new('', '')~loadPackage('smreq.cls')~name~right(9); syntax: return condition('o')~code condition('o')~message")
  r~setSecurityManager(mgr)
  say label r~call
  say label "events:" mgr~count
  return

::class Mgr
::method init
  expose replacement count
  use arg replacement
  count = 0
::method count
  expose count
  return count
::method unknown
  expose replacement count
  use arg name, args
  info = args[1]
  if name \= "REQUIRES" then return 0
  count += 1
  if replacement == "" then return 0
  if replacement == "-" then info~remove('NAME')
  else info['NAME'] = replacement
  return 1
