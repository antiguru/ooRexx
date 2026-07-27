/* extracted from SysFormatMessage::test_substitutions */
::routine main public
  self~assertEquals("abc", SysFormatMessage("&1", "abc"))
  self~assertEquals("abc", SysFormatMessage("&1", ("abc", "def")))
  self~assertEquals("abc + def", SysFormatMessage("&1 + &2", ("abc", "def")))
  self~assertEquals("def", SysFormatMessage("&2", ("abc", "def")))
  self~assertEquals("def + abc", SysFormatMessage("&2 + &1", ("abc", "def")))
  self~assertEquals("def & abc", SysFormatMessage("&2 & &1", ("abc", "def")))
  self~assertEquals("abc +  + def", SysFormatMessage("&1 + &2 + &3", ("abc",, "def")))
  self~assertEquals("123", SysFormatMessage("123", ("abc", "def")))
  self~assertEquals("&0 abc", SysFormatMessage("&0 &1", "abc"))
  self~assertEquals("", SysFormatMessage("&1", .array~new))
  self~assertSame("", SysFormatMessage("", ""))
  self~assertSame("&&&", SysFormatMessage("&&1&", "&"))
  self~assertSame(" leading...", SysFormatMessage(" leading&1", "..."))
  self~assertSame(">>trailing  ", SysFormatMessage("&1trailing  ", ">>"))
  self~assertSame(" leading-trailing ", SysFormatMessage(" leading&1trailing ", "-"))
  self~assertSame("9 8 7 6 5 4 3 2 1 &0", SysFormatMessage("&9 &8 &7 &6 &5 &4 &3 &2 &1 &0", .Array~new(9)~fill(0)~allIndexes))

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
