/* extracted from whiteSpace::test_TABs_STRIP */
::routine main public
    TAB  ="09"x
    TAB2 =TAB||TAB

    PLANK=" "
    PLANK2=PLANK||PLANK

    str0a=plank2 "This" plank "is" tab "a" plank2 "test." plank2
    str0b=tab2   "This" plank "is" tab "a" plank2 "test." tab2

    str1="This" plank "is" tab "a" plank2 "test."

    str2a="This" plank "is" tab "a" plank2 "test." plank2
    str2b="This" plank "is" tab "a" plank2 "test." tab2

    str3a=plank2 "This" plank "is" tab "a" plank2 "test."
    str3b=tab2   "This" plank "is" tab "a" plank2 "test."


    self~assertSame(str1, strip(str0a))
    self~assertSame(str1, strip(str0a,'B'))
    self~assertSame(str2a, strip(str0a,'L'))
    self~assertSame(str3a, strip(str0a,'T'))

    self~assertSame(str1, strip(str0b))
    self~assertSame(str1, strip(str0b,'B'))
    self~assertSame(str2b, strip(str0b,'L'))
    self~assertSame(str3b, strip(str0b,'T'))


   -- starting with 3.2 TAB chars are treated like blanks
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
