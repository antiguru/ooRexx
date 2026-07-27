/* extracted from CALL::test_symbol */
::routine main public
  -- search order is internal labels, built-in functions, external functions
  call label; self~assertSame("internal", result)
  call arg; self~assertSame("internal", result) -- should not call ARG built-in
  call upper; self~assertSame("internal", result) -- should not call UPPER built-in
  call digits; self~assertSame(9, result) -- should call DIGITS built-in, not "digits" label

  -- namespace-prefixed call target
  pkg = .Package~new("", ("::routine call public", "return 'routine'"))
  .context~package~addPackage(pkg, "space")
  call space:call; self~assertSame("routine", .CALL.trap)

  -- now repeat for function syntax; not really a CALL test, but still ..
  self~assertSame("internal", label())
  self~assertSame("internal", arg())
  self~assertSame("internal", upper())
  self~assertSame(9, digits())
  self~assertSame("routine", space:call())

  return

  label: arg: "UPPER": "digits": return "internal"

::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
