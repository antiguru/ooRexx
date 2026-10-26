/* A ::CONSTANT expression runs as a method of its own, named ::CONSTANT,
   on its class: a METHOD level stopped on the directive, above the level
   installing the package, whose executable is a Method that is not the
   accessor. */
call stream 'cfcall.rex', 'c', 'open write replace'
call lineout 'cfcall.rex', 'say .k~s~makestring("l", ",")'
call lineout 'cfcall.rex', '::class k'
call lineout 'cfcall.rex', '::constant s (.k~frames)'
call lineout 'cfcall.rex', '::method frames class'
call lineout 'cfcall.rex', '  a = .array~new'
call lineout 'cfcall.rex', '  do f over .context~stackframes'
call lineout 'cfcall.rex', '    a~append(f~type filespec("name", f~name) f~line)'
call lineout 'cfcall.rex', '  end'
call lineout 'cfcall.rex', '  return a'
call lineout 'cfcall.rex'
say .c~s~makestring('l', ',')
call cfcall.rex
say .c~j
e = .c~e
say e~class e~isGuarded e~isPrivate e~isProtected e~isPackage e~isAbstract -
  e~isConstant e~isAttribute
say (e~package == .context~package) (e~scope == .c) (e == .c~e) -
  (.c~method('E') == e) e~source~items e~source[1]
::class c public
::constant s (.c~frames)
::constant j (.context~executable~class .context~name .context~args~items -
  (.context~executable == .c~method('J')))
::method frames class
  a = .array~new
  do f over .context~stackframes
    a~append(f~type filespec('name', f~name) f~line)
  end
  return a
::constant e (.context~executable)
