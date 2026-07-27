/* extracted from StackFrame::testNativeCodeFrame */
::routine main public
  test = .array~of(1,2)

  comparator = .ContextComparator~new
  test~sortwith(comparator)

  -- the sortwith method should be one back
  frame = comparator~stackframes[2]
  self~assertSame("METHOD", frame~type)
  self~assertTrue(.array~of(comparator)~equivalent(frame~arguments))
  self~assertSame("SORTWITH", frame~name)
  self~assertSame(.Array~method("SORTWITH"), frame~executable)
  self~assertSame(.nil, frame~line)
  self~assertSame(test, frame~target)
  self~assertSame(frame~traceLine, frame~string)
  self~assertSame(frame~traceLine, frame~makestring)

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
