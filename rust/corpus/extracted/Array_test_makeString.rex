/* extracted from Array::test_makeString */
::routine main public

  nl=.ooRexxUnit.line.separator     -- get line separator
  a=.array~new
  self~assertEquals("", a~makestring)
  self~assertEquals("", a~makestring("c"))
  self~assertEquals("", a~makestring("C"))
  self~assertEquals("", a~makestring("l"))
  self~assertEquals("", a~makestring("L"))
  self~assertEquals("", a~makestring(,"~"))


  a=.array~of("1v")
  str1="1v"
  self~assertEquals(str1, a~makestring)
  self~assertEquals(str1, a~makestring("c"))
  self~assertEquals(str1, a~makestring("C"))
  self~assertEquals(str1, a~makestring("l"))
  self~assertEquals(str1, a~makestring("L"))
  self~assertEquals(str1, a~makestring(,"~"))


  a=.array~of("1v", "2v", "3v")
  str1="1v2v3v"
  str2=str1~insert(nl,4)~insert(nl,2)
  str3=str1~insert("~",4)~insert("~",2)

  self~assertEquals(str2, a~makestring)
  self~assertEquals(str1, a~makestring("c"))
  self~assertEquals(str1, a~makestring("C"))
  self~assertEquals(str2, a~makestring("l"))
  self~assertEquals(str2, a~makestring("L"))
  self~assertEquals(str3, a~makestring(,"~"))


  a=.array~of("1v",, "2v",, "3v")
  str1="1v2v3v"
  str2=str1~insert(nl,4)~insert(nl,2)
  str3=str1~insert("~",4)~insert("~",2)

  self~assertEquals(str2, a~makestring)
  self~assertEquals(str1, a~makestring("c"))
  self~assertEquals(str1, a~makestring("C"))
  self~assertEquals(str2, a~makestring("l"))
  self~assertEquals(str2, a~makestring("L"))
  self~assertEquals(str3, a~makestring(,"~"))


  a=.array~of(.set~new,, "2v",, .directory~new)
  str1="2v"
  str2="a Set"||nl||str1||nl||"a Directory"
  str3="a Set~"str1"~a Directory"

  self~assertEquals(str2, a~makestring)
  self~assertEquals("a Set"||str1||"a Directory", a~makestring("c"))
  self~assertEquals("a Set"||str1||"a Directory", a~makestring("C"))
  self~assertEquals(str2, a~makestring("l"))
  self~assertEquals(str2, a~makestring("L"))
  self~assertEquals(str3, a~makestring(,"~"))


  a=.array~new
  a[1,1]="11v";a[2,1]="12v";a[1,2]="21v";a[2,2]="22v"
  str1="11v12v21v22v"
  str2=str1~insert(nl,9)~insert(nl,6)~insert(nl,3)
  str3=str1~insert("~",9)~insert("~",6)~insert("~",3)

  self~assertEquals(str2, a~makestring)
  self~assertEquals(str1, a~makestring("c"))
  self~assertEquals(str1, a~makestring("C"))
  self~assertEquals(str2, a~makestring("l"))
  self~assertEquals(str2, a~makestring("L"))
  self~assertEquals(str3, a~makestring(,"~"))

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
