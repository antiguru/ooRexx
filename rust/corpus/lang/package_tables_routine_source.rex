-- A package's class and routine tables answer what a StringTable answers, and
-- Routine~new over a source with directives installs them in its own package.
say .context~package~classes~items
routine = .Routine~new("Testing", "return 123")
p = routine~package
say p~routines~items p~publicRoutines~items p~classes~items p~name
src = .array~of("::routine a1", "  return 1", "::routine a2 public", "  return 2")
r = .Routine~new("Testing", src)
p = r~package
rs = p~routines
say rs~items rs~hasIndex('A1') rs['A1']~isA(.routine) rs['A1']~call rs['A2']~call p~publicRoutines~items p~publicRoutines~hasIndex('A2')
say r~package~name
src = .array~of("::class a1", "::class a2 public", "::method hi", "  return 'hi' self~class~id")
p = .Routine~new("Cls", src)~package
cs = p~classes
say cs~items cs~hasIndex('A1') cs['A1']~isA(.class) cs['A2']~id p~publicClasses~items p~publicClasses~hasIndex('A2')
say cs['A2']~new~hi
say .Routine~new('T', .array~of('::class a1'))~package~classes~hasIndex('A1')
r = .Routine~new("Calls", .array~of("return helper() b()", "::routine b", "return 'b'"))
say r~call
say cs~allIndexes~makestring('l', ',') cs~makeArray~items
do n over cs; say 'over' n; end
exit
helper: return 'h'
::routine helper
  return 'helper'
