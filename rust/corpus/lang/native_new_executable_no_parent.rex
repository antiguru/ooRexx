/* A method or routine that NewMethod or NewRoutine builds from source has a
   package with no parent: a routine or class only the caller's package
   defines is not found in it. .Method~new and .Routine~new inherit the
   caller's package. */
t = .T~new
m = t~nm('M', 'return helper()')
say m~class m~package~name m~package~routines~items m~package~importedPackages~items
call trym m
r = t~nr('R', 'return helper()')
say r~class r~package~name r~package~routines~items r~package~importedPackages~items
call tryr r
say 'class method:' t~go(t~nm('M3', 'return .t'))
say 'class routine:' t~nr('R3', 'return .t')~call
nl = '0a'x
m4 = t~nm('M4', 'x = 1' || nl || 'return inner() x' || nl || '::routine inner' || nl || 'return "inner ran"')
say 'method directives:' t~go(m4) m4~source~items m4~package~name m4~package~routines~items
say m4~source~toString(, '|')
r4 = t~nr('R4', 'say "r4 line 1"' || '0d0a'x || 'return inner2()' || nl || '::routine inner2' || nl || 'return .context~package~name .context~name')
say 'routine directives:' r4~call r4~source~items r4~package~name
say r4~source~toString(, '|')
say 'rexx method:' t~go(.method~new('M2', 'return helper()'))
say 'rexx routine:' .routine~new('R2', 'return helper()')~call
exit
trym: procedure
  use arg m
  signal on syntax
  say 'method:' .T~new~go(m)
  return
syntax: say 'method' condition('o')~code condition('o')~message; return
tryr: procedure
  use arg r
  signal on syntax name rsyntax
  say 'routine:' r~call
  return
rsyntax: say 'routine' condition('o')~code condition('o')~message; return
::routine helper
  return 'helper ran'
::class T public
::method nm external "LIBRARY orxmethod TestNewMethod"
::method nr external "LIBRARY orxmethod TestNewRoutine"
::method go
  use arg m
  return self~run(m)
