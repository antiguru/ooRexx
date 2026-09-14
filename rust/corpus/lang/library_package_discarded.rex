/* A package whose translation raised keeps its settings and none of the
   routines, unattached methods, resources or prolog it declared before the
   refusal: a library method it bound still names it as its package, and
   nothing it declared is found through it, as a context either. */
early = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Parse')
say 'pk.cls' try('pk.cls')
pkg = early~package
say 'bound package' show(early)
say 'findRoutine' pkg~findRoutine('PKR')
say 'routines[PKR]' pkg~routines['PKR']
say 'publicRoutines[PKR]' pkg~publicRoutines['PKR']
n = 0
do name over pkg~routines
  n = n + 1
end
say 'routines over' n
say 'definedMethods[UM]' pkg~definedMethods['UM']
say 'resource RES' pkg~resource('RES')
say 'prolog' pkg~prolog
say 'digits' pkg~digits
say 'Package~new context' newpackage('call pkr', early)
say 'newFile context' newfile('rf.cls', early)

later = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')
say 'pk2.cls' try('pk2.cls')
pkg2 = later~package
say 'bound package' show(later)
say 'findRoutine' pkg2~findRoutine('PKR2')
say 'routines[PKR2]' pkg2~routines['PKR2']
say 'prolog' pkg2~prolog
say 'Package~new context' newpackage('call pkr2', later)
exit

try: procedure
  use arg name
  signal on syntax name raised
  p = .context~package~loadPackage(name)
  return 'loaded'
raised:
  return 'raised' condition('O')~code

newpackage: procedure
  use arg source, context
  signal on syntax name raised
  p = .Package~new('fromsource', source, context)
  return 'answered'
raised:
  return 'raised' condition('O')~code

newfile: procedure
  use arg file, context
  signal on syntax name raised
  r = .Routine~newFile(file, context)
  return 'answered' r~call
raised:
  return 'raised' condition('O')~code

show: procedure
  use arg x
  p = x~package
  if p == .nil then return 'nil'
  return p~name~substr(p~name~lastpos('/') + 1)
