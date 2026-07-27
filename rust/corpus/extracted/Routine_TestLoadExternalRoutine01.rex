/* extracted from Routine::TestLoadExternalRoutine01 */
::routine main public
  routine = .routine~loadExternalRoutine("SQRT", "LIBRARY rxmath RxCalcSqrt")
  self~assertTrue(routine~isA(.routine))
  self~assertSame(2, routine~call(4))

  routine = .routine~loadExternalRoutine("RxCalcSqrt", "LIBRARY rxmath")
  self~assertTrue(routine~isA(.routine))
  self~assertSame(2, routine~call(4))

  routine = .routine~loadExternalRoutine("RxCalcSqrt", "LIBRARY rxmath not_one_we_have")
  self~assertSame(.nil, routine)

  routine = .routine~loadExternalRoutine("RxCalcSqrt", "LIBRARY not_one_we_have")
  self~assertSame(.nil, routine)

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
