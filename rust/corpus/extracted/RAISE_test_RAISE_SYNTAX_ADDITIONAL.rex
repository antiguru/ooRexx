/* extracted from RAISE::test_RAISE_SYNTAX_ADDITIONAL */
::routine main public
  signal on syntax            -- intercept syntax error
  a.1="1"
  a.2="'anImportantValue'"
  a.3="3B"
  a.0=3

  a=.array~of(a.1, a.2, a.3)

  raise SYNTAX 40.12 additional (a)

syntax:
  co=condition("Object")      -- get condition object
  rgf_Sigl=SIGL               -- save signal line number

  self~assertNotNull(co~additional)

  a=co~additional    -- get additional array
  self~assertEquals(a.1, a[1])
  self~assertEquals(a.2, a[2])
  self~assertEquals(a.3, a[3])

  self~assertEquals(40.12, co~code)

  self~assertEquals("SYNTAX", co~condition)

  self~assertEquals("", co~description)

  self~assertEquals("Incorrect call to routine.", co~errortext)

  self~assertEquals("SIGNAL", co~instruction)

  self~assertEquals(a.1" argument "a.2" must be a whole number; found "||'"'a.3||'".', co~message)

  self~assertEquals(rgf_Sigl, co~position)

  parse source . . full_path
  self~assertEquals(full_path, co~program)

  self~assertFalse(co~propagated)

  self~assertEquals(40, co~rc)

  self~assertNull(co~result)

  self~assertNull(co~source)



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
