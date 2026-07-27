/* extracted from StackFrame::testCompileFrame */
::routine main public

  signal on syntax
  .Method~new("", "~~")
  -- we should never reach this
  self~expectSyntax(35.1)              -- Incorrect expression detected at "~~"
  return

  syntax:
  info = grabContextInfo(.Context, .line, condition("o")["STACKFRAMES"])

  -- in contrast to .Context~stackFrames(), which returns an Array of stack frames,
  -- the condition object's "STACKFRAMES" entry returns a List of stack frames
  -- thus we have to use firstItem() here, using [1] won't work
  frame = info~stackFrames~firstItem
  -- COMPILE frames do not have arguments, nor name, executable, target, or context
  -- here, line is 1
  self~assertSame("COMPILE", frame~type)
  self~assertSame(0, frame~arguments~items)
  self~assertSame("", frame~name)
  self~assertNull(frame~executable)
  self~assertSame(1, frame~line)
  self~assertNull(frame~target)
  self~assertSame(frame~traceLine, frame~string)
  self~assertSame(frame~traceLine, frame~makeString)

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
