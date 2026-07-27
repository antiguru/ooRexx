/* extracted from caselessWordPos::test_WORDPOS_caseless */
::routine main public

   self~assertEquals(3, 'now is the time'~caselessWordPos('the'))
   self~assertEquals(3, 'now is the time'~caselessWordPos('The'))
   self~assertEquals(2, 'now is the time'~caselessWordPos('is tHE'))
   self~assertEquals(2, 'now is the time'~caselessWordPos('Is ThE'))
   self~assertEquals(2, 'now is the time'~caselessWordPos('is   the'))
   self~assertEquals(2, 'now is the time'~caselessWordPos('iS   ThE'))
   self~assertEquals(0, 'now is   the time'~caselessWordPos('is    time '))
   self~assertEquals(2, 'To be or not to be'~caselessWordPos('be'))
   self~assertEquals(2, 'To be or not to be'~caselessWordPos('Be'))
   self~assertEquals(6, 'To be or not to be'~caselessWordPos('be',3))
   self~assertEquals(6, 'To be or not to be'~caselessWordPos('bE',3))

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
