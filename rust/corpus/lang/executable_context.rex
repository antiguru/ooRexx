/* The context argument `Routine~newFile`, `Method~newFile` and `Package~new`
   take, and which package the loaded text then resolves names through.

   `newFile` accepts the string `"PROGRAMSCOPE"` as well as a Method, Routine
   or Package object, and the string means what an absent argument means: the
   caller's own package. A Method or Routine hands over the package it belongs
   to, which for a primitive is the REXX package and defines no ::ROUTINE of
   this program's.

   `Package~new` takes the same three objects and not the string, and only its
   source form takes one at all: the file form never consumes that argument, so
   it reaches `Object~init` and is an error there whenever the load itself
   raises nothing. `Object~init` is what refuses a surplus argument to either
   form, after the package is built and its prologue has run. */

call lineout 'body.rex', "return helper()"
call stream 'body.rex', 'c', 'close'
call lineout 'plain.rex', "say helper()"
call stream 'plain.rex', 'c', 'close'
call lineout 'given.rex', "say helper()"
call stream 'given.rex', 'c', 'close'
call lineout 'loads.rex', "nop"
call stream 'loads.rex', 'c', 'close'

say 'default   ' .Routine~newFile('body.rex')~call
say 'scope     ' .Routine~newFile('body.rex', 'PROGRAMSCOPE')~call
say 'caseless  ' .Routine~newFile('body.rex', 'programscope')~call

elsewhere = .Package~new('c2', .array~of("::routine helper public", "  return 'from c2'"))
say 'package   ' .Routine~newFile('body.rex', elsewhere)~call
say 'routine   ' .Routine~newFile('body.rex', .context~package~findRoutine('HELPER'))~call
say 'method    ' .Method~newFile('body.rex', elsewhere)~class~id

say 'native    ' attempt(.Array~method('APPEND'))
say 'unknown   ' attempt('NONSENSE')
say 'nil       ' attempt(.nil)
say 'array     ' attempt(.array~of(1))
say 'class     ' attempt(.Array)

built = .Package~new('src', .array~of("say 'prologue' helper()"), .context~package)
say 'source    ' built~name

say 'string    ' newpackage(.array~of("say helper()"), 'PROGRAMSCOPE')
say 'too many  ' newpackage(.array~of("nop"), .context~package, 'X')
say 'no parent ' nocontext()
say 'file      ' filepackage('plain.rex', 0)
say 'file+ctx  ' filepackage('given.rex', 1)
say 'loads     ' filepackage('loads.rex', 0)
say 'loads+ctx ' filepackage('loads.rex', 1)

exit 0

attempt: procedure
  use arg context
  signal on syntax name failed
  return .Routine~newFile('body.rex', context)~call
failed:
  raised = condition('O')
  return raised['CODE'] raised['MESSAGE']

newpackage: procedure
  use arg source, context, extra
  signal on syntax name refused
  if arg(3, 'e') then return .Package~new('n', source, context, extra)~name
  return .Package~new('n', source, context)~name
refused:
  raised = condition('O')
  return raised['CODE'] raised['MESSAGE']

nocontext: procedure
  signal on syntax name absent
  return .Package~new('n', .array~of("say helper()"))~name
absent:
  raised = condition('O')
  return raised['CODE'] raised['MESSAGE']

filepackage: procedure
  use arg file, given
  signal on syntax name missing
  if given then return built(.Package~new(file, , .context~package))
  return built(.Package~new(file))
missing:
  raised = condition('O')
  return raised['CODE'] raised['MESSAGE']

built: procedure
  use arg package
  return 'a' package~class~id 'holding' package~source~items 'lines'

::routine helper
  return 'from main'
