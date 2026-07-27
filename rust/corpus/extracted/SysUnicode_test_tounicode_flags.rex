/* extracted from SysUnicode::test_tounicode_flags */
::routine main public
  -- all valid flags should be recognized, with PRECOMPOSED and COMPOSITE
  -- being mutually exclusive
  flags = "Composite err_invalid_chars UseGlyphChars"
  self~assertSame(0, SysToUnicode("A", 850, flags, s.))
  flags = "ERR_INVALID_CHARS PRECOMPOSED USEGLYPHCHARS"
  self~assertSame(0, SysToUnicode("A", 850, flags, s.))
  -- in addition we have one undocumented alias
  self~assertSame(0, SysToUnicode("A", 850, "ERR_INVALID", s.))

  -- COMPOSITE

  -- three flags allowed together
  self~assertSame(0, SysToUnicode("A", 850, "COMPOSITE ERR_INVALID_CHARS USEGLYPHCHARS", s.))
  -- COMPOSITE cannot be used together with PRECOMPOSED
  -- PRECOMPOSED and COMPOSITE are mutually exclusive
  self~assertSame(1004, SysToUnicode("A", 850, "COMPOSITE PRECOMPOSED", s.))
  -- but no error is raised for code pagr UTF8, so we run this as "known bug"
  self~assertSame(0, SysToUnicode("A", "UTF8", "COMPOSITE PRECOMPOSED", s.), "tracker bug #n/a MultiByteToWideChar COMPOSITE PRECOMPOSED should fail")
  -- to do: more COMPOSITE tests

  -- PRECOMPOSED

  -- three flags allowed together
  self~assertSame(0, SysToUnicode("A", 850, "precomposed err_invalid_chars useglyphchars", s.))
  -- to do: more PRECOMPOSED tests


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
