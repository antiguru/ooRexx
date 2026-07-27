/* extracted from comparisonOperators::test_nil */
::routine main public
  signal on nostring

  -- verify the various tests against .nil.
  test = .nil~string   -- test against the string version of .nil to make sure we don't get false positives

  self~assertFalse(test = .nil)
  self~assertTrue(test \= .nil)
  self~assertFalse(test == .nil)
  self~assertTrue(test \== .nil)
  self~assertTrue(test <> .nil)
  self~assertTrue(test >< .nil)
  self~assertFalse(test > .nil)
  self~assertFalse(test < .nil)
  self~assertFalse(test >= .nil)
  self~assertFalse(test <= .nil)
  self~assertFalse(test \> .nil)
  self~assertFalse(test \< .nil)
  self~assertFalse(test << .nil)
  self~assertFalse(test >> .nil)
  self~assertFalse(test >>= .nil)
  self~assertFalse(test <<= .nil)
  self~assertFalse(test \<< .nil)
  self~assertFalse(test \>> .nil)
  -- because of the internal classes, test also with integer and decimal numbers
  test = 120

  self~assertFalse(test = .nil)
  self~assertTrue(test \= .nil)
  self~assertFalse(test == .nil)
  self~assertTrue(test \== .nil)
  self~assertTrue(test <> .nil)
  self~assertTrue(test >< .nil)
  self~assertFalse(test > .nil)
  self~assertFalse(test < .nil)
  self~assertFalse(test >= .nil)
  self~assertFalse(test <= .nil)
  self~assertFalse(test \> .nil)
  self~assertFalse(test \< .nil)
  self~assertFalse(test << .nil)
  self~assertFalse(test >> .nil)
  self~assertFalse(test >>= .nil)
  self~assertFalse(test <<= .nil)
  self~assertFalse(test \<< .nil)
  self~assertFalse(test \>> .nil)

  test = 120.1

  self~assertFalse(test = .nil)
  self~assertTrue(test \= .nil)
  self~assertFalse(test == .nil)
  self~assertTrue(test \== .nil)
  self~assertTrue(test <> .nil)
  self~assertTrue(test >< .nil)
  self~assertFalse(test > .nil)
  self~assertFalse(test < .nil)
  self~assertFalse(test >= .nil)
  self~assertFalse(test <= .nil)
  self~assertFalse(test \> .nil)
  self~assertFalse(test \< .nil)
  self~assertFalse(test << .nil)
  self~assertFalse(test >> .nil)
  self~assertFalse(test >>= .nil)
  self~assertFalse(test <<= .nil)
  self~assertFalse(test \<< .nil)
  self~assertFalse(test \>> .nil)
  return

  nostring:
  self~assertTrue(.false, "Unexpected NOSTRING condition raised at" sigl)

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
