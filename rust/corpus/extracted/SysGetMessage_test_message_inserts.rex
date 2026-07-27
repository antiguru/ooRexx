/* extracted from SysGetMessage::test_message_inserts */
::routine main public
  -- 678: Argument &1 must be in the range &2 to &3; found "&4".
  self~assertSame('Argument  must be in the range  to ; found "".', SysGetMessage(678))
  self~assertSame('Argument 1 must be in the range  to ; found "".', SysGetMessage(678, , 1))
  self~assertSame('Argument 1 must be in the range 2 to ; found "4".', SysGetMessage(678, , 1, 2, , 4))
  self~assertSame('Argument 1 must be in the range 2 to 3; found "4".', SysGetMessage(678, , 1, 2, 3, 4))
  self~assertSame('Argument  must be in the range  to 3; found "4".', SysGetMessage(678, , , , 3, 4, 5))

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
