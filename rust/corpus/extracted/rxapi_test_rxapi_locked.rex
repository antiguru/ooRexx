/* extracted from rxapi::test_rxapi_locked */
::routine main public
  -- as we're successfully running Rexx, rxapi should be up and running
  -- starting rxapi a second time should fail with this expected output:
  -- rxapi: lockfile path is /path/to/.ooRexx-v.r.m-bb-user.lock
  -- rxapi: lockfile is locked by another rxapi instance; exiting
  rexx = .RexxInfo~executable
  if rexx == .nil then return -- we don't know which rxapi to run
  -- rxapi is in the same directory as rexx
  rxapi = .File~new("rxapi", rexx~parent)
  output = .Array~new
  address "path" rxapi with output using (output)
  self~assertSame(2, output~items)
  expected = "rxapi: lockfile path is"
  self~assertTrue(output[1]~startsWith(expected), output[1] "should start with" expected)
  expected = "rxapi: lockfile is locked"
  self~assertTrue(output[2]~startsWith(expected), output[2] "should start with" expected)


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
