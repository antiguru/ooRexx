/* extracted from CALL::test_8 */
::routine main public
   x=0; Call 0001
   self~assertSame(x, 10)
   return

   0003: x=x+1; Call 0004; Return /* 0003 */
   0009: x=x+1; Call 0010; Return /* 0004 */
   0008: x=x+1; Call 0009; Return /* 0001 */
   0005: x=x+1; Call 0006; Return /* 0010 */
   0004: x=x+1; Call 0005; Return /* 0007 */
   0007: x=x+1; Call 0008; Return /* 0005 */
   0001: x=x+1; Call 0002; Return /* 0009 */
   0010: x=x+1; Call 0011; Return /* 0006 */
   0006: x=x+1; Call 0007; Return /* 0002 */
   0002: x=x+1; Call 0003; Return /* 0008 */
   0011: Return

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
