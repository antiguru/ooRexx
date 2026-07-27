/* extracted from caselessContainsWord::test1_WORDPOS_caseless */
::routine main public

   self~assertEquals(1, 'now is the time'~caselessContainsWord('the'))
   self~assertEquals(1, 'now is the time'~caselessContainsWord('The'))
   self~assertEquals(1, 'now is the time'~caselessContainsWord('is tHE'))
   self~assertEquals(1, 'now is the time'~caselessContainsWord('Is ThE'))
   self~assertEquals(1, 'now is the time'~caselessContainsWord('is   the'))
   self~assertEquals(1, 'now is the time'~caselessContainsWord('iS   ThE'))
   self~assertEquals(0, 'now is   the time'~caselessContainsWord('is    time '))
   self~assertEquals(1, 'To be or not to be'~caselessContainsWord('be'))
   self~assertEquals(1, 'To be or not to be'~caselessContainsWord('Be'))
   self~assertEquals(1, 'To be or not to be'~caselessContainsWord('be',3))
   self~assertEquals(1, 'To be or not to be'~caselessContainsWord('bE',3))

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
