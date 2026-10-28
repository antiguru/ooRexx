e = .c~e
say e~class e~isGuarded e~isPrivate e~isProtected e~isPackage e~isAbstract e~isConstant e~isAttribute
say e~package~name~right(6) (e~package == .context~package)
s = e~source
say s~class s~items
do l over s; say '['l']'; end
say e~scope
say (e == .c~e)
say .c~method('E') == e
::class c
::constant e (.context~executable)
