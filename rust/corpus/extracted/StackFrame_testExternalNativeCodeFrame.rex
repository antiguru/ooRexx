/* extracted from StackFrame::testExternalNativeCodeFrame */
::routine main public
  signal on syntax

  s = .stream~new("test.dat")
  s~charin("abc")

  exit
  syntax:

  o = condition('o')

  frame = o~stackframes~firstItem
  self~assertSame("METHOD", frame~type)
  self~assertTrue(.array~of("abc")~equivalent(frame~arguments))
  self~assertSame("CHARIN", frame~name)
  self~assertSame(.Stream~method("CHARIN"), frame~executable)
  self~assertSame(.nil, frame~line)
  self~assertSame(s, frame~target)
  self~assertSame(frame~traceLine, frame~string)
  self~assertSame(frame~traceLine, frame~makestring)

-- [bugs:#1751] INTERPRET wipes out its parent's StackFrame name
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
