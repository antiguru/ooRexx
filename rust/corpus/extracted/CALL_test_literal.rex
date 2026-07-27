/* extracted from CALL::test_literal */
::routine main public
  -- if name is a literal string any internal labels are bypassed
  call ""; self~assertSame("routine", result) -- should not call internal "" label
  call ''b; self~assertSame("routine", result) -- binary nullstring
  call "label"; self~assertSame("routine", result) -- always case-insensitive
  call "Label"; self~assertSame("routine", result) -- always case-insensitive
  call "LABEL"; self~assertSame("routine", result) -- should not call internal label
  call "arg"; self~assertSame("routine", result) -- should neither call ARG built-in nor "arg" label
  call "ARG"; self~assertSame(0, result) -- should call ARG built-in
  call '41 52 47'x; self~assertSame(0, result) -- "ARG" as hex string

  -- repeat for function syntax
  self~assertSame("routine", ""())
  self~assertSame("routine", ''b())
  self~assertSame("routine", "label"())
  self~assertSame("routine", "Label"())
  self~assertSame("routine", "arg"())
  self~assertSame(0, "ARG"())
  self~assertSame(0, '41 52 47'x())

  return

  "": label: arg: "arg": return "internal"

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
