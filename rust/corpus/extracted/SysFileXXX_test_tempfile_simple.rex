/* extracted from SysFileXXX::test_tempfile_simple */
::routine main public
  s = .File~separator
  self~assertSame("?", template("?"))
  self~assertSame("??", template("??"))
  self~assertSame("???", template("???"))
  self~assertSame("????", template("????"))
  self~assertSame("?????", template("?????"))
  self~assertSame("??????", template("??????"))
  self~assertSame("???????", template("???????"))
  self~assertSame("????????", template("????????"))
  self~assertSame("?????????", template("?????????"))

  self~assertSame("*", template("*", "*"))
  self~assertSame("?x", template("?x"))
  self~assertSame("-?-", template("-?-"))
  self~assertSame("file???", template("file???"))
  self~assertSame("test.dat", template("test.dat", "."))

  self~assertSame("dir"s"file?", template("dir"s"file?"))
  self~assertSame("dir?"s"file?", template("dir?"s"file?"))
  self~assertSame(".dir."s".file.", template(".dir."s".file.", "."))
  self~assertSame("*********", template("*********", "*"))
  return

  template: procedure
  use arg template, filler
  if arg(2, "omitted") then
    return SysTempFileName(template)~translate(, "0123456789", "?")~right(template~length)
  else
    return SysTempFileName(template, filler)~translate(, "0123456789", filler)~right(template~length)

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
