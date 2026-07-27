/* extracted from Class::test_activate */
::routine main public
  -- create a directory that the loaded package can store results in
  .local~class.testgroup = .directory~new
  -- now load a package to trigger the activate tests
  .context~package~loadPackage("class.testgroup.cls")
  -- give an assertion failure if something was detected
  self~assertTrue(.local~class.testgroup~assertFail \= .true, .local~class.testgroup~assertFailReason)
  -- and verify that the activate methods were even called
  self~assertTrue(.local~class.testgroup~class1 == .true)
  self~assertTrue(.local~class.testgroup~class2 == .true)
  self~assertTrue(.local~class.testgroup~class3 == .true)

  .local~remove("class.testgroup"~upper)

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
