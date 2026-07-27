/* extracted from LabelOption::test_DO_lable_works_with_interpret */
::routine main public
   str="i=0                                                                    ;" -
   "DO label aha1                                                              ;" -
   "   i=i+1                                                                   ;" -
   "   DO label aha2                                                           ;" -
   "      leave aha2                                                           ;" -
   "      i = 9                                                                ;" -
   "   end                                                                     ;" -
   "   i=i+1                                                                   ;" -
   "END aha1                                                                   ;" -

   interpret str

   self~assertNotEquals(0, i, "i must be incremented")
   self~assertNotEquals(10, i, "Leave aha2 did not work")
   self~assertEquals(2, i, "Do loops with label must leave i == 2")


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
