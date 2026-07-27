/* extracted from SysUnicode::test_fromunicode_flags */
::routine main public
  -- all valid flags should be recognized, just ERR_INVALID_CHARS is special
  flags = "CompositeCheck DEFAULTCHAR discardNS sepChars NO_best_fit_chars"
  self~assertSame(0, SysFromUnicode(.Unicode~A, 850, flags, , s.))
  -- in addition we have this undocumented alias
  self~assertSame(0, SysFromUnicode(.Unicode~A, 850, "NO_BEST_FIT", , s.))

  -- ERR_INVALID_CHARS is valid with codepages UTF8 or 54936 only
  self~assertSame(0, SysFromUnicode(.Unicode~A, "utf8", "Err_Invalid_Chars", , s.), -
   "UTF8 is known to fail prior to Windows 10 2018")
  -- in addition we have this undocumented alias
  self~assertSame(0, SysFromUnicode(.Unicode~A, "utf8", "ERR_INVALID", , s.))


  -- NO_BEST_FIT_CHARS

  -- &copy; in 437 is "best-fit" character "c"
  self~assertSame("0 c", SysFromUnicode(.Unicode~copy, 437, , , s.) s.!text)
  -- with NO_BEST_FIT_CHARS it should return "?" with !USEDDEFAULTCHAR .true
  self~assertSame("0 ? 1", SysFromUnicode(.Unicode~copy, 437, "NO_BEST_FIT_CHARS", , s.) s.!text s.!useddefaultchar)

  -- ERR_INVALID_CHARS

  -- valid with codepages UTF8 or 54936 only
  -- otherwise fails with 1004 "Invalid flags"
  self~assertSame(1004, SysFromUnicode(.Unicode~A, 437, "ERR_INVALID_CHARS", , s.))

  -- for codepage UTF8 it fails with 1113 "No mapping for the Unicode character
  -- exists in the target multi-byte code page" for an invalid input character
  self~assertSame(1113, SysFromUnicode(.Unicode~DFFF, "UTF8", "ERR_INVALID_CHARS", , s.))
  self~assertSame(0, SysFromUnicode(.Unicode~A, "UTF8", "ERR_INVALID_CHARS", , s.))

  -- COMPOSITECHECK DEFAULTCHAR combo

  -- &infin; is available in 437, but "best-fit" approximated in 1252 with digit "8" (crazy!)
  self~assertSame("0 EC 0", SysFromUnicode(.Unicode~infin, 437, , , s.) s.!text~c2x s.!useddefaultchar)
  self~assertSame("0 8 0", SysFromUnicode(.Unicode~infin, 1252, , , s.) s.!text s.!useddefaultchar)
  -- This test sets NO_BEST_FIT_CHARS so that infin is translated to the
  -- default character instead, and sets COMPOSITECHECK DEFAULTCHAR for a
  -- unique default character.  !USEDDEFAULTCHAR should be .true
  self~assertSame("0 _ 1", SysFromUnicode(.Unicode~infin, 1252, -
    "no_best_fit_chars compositecheck defaultchar", "_", s.) s.!text s.!useddefaultchar)

  -- COMPOSITECHECK DISCARDNS combo
  -- tests to be done

  -- COMPOSITECHECK and COMPOSITECHECK SEPCHARS combo

  -- A composite like A plus combining diaeresis should become a single character
  -- if either of above flag/combo is specified.  And two characters else.
  composite = .Unicode~A || .Unicode~uml
  self~assertSame("0 1", SysFromUnicode(composite, 1252, 'COMPOSITECHECK', , s.) s.!text~length)
  self~assertSame("0 1", SysFromUnicode(composite, 1252, 'COMPOSITECHECK SEPCHARS', , s.) s.!text~length)
  self~assertSame("0 2", SysFromUnicode(composite, 1252, , , s.) s.!text~length)

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
