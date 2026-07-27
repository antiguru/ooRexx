/* extracted from SysFileTree::test_name_UNC */
::routine main public
  parse source . . this
  -- thisFile is in thisDir, so SysFileTree should find it
  thisFile = .File~new(this~changeStr(':', '$'), '\\localhost')
  thisDir = .File~new("*", thisFile~parentFile)
  self~assertSame(0, SysFileTree(thisDir~absolutePath, files., "fo"), "SysFileTree("thisDir~absolutePath") should give rc=0")
  self~assertTrue(files.0 > 0, "SysFileTree should have found at least one file in" thisDir~absolutePath)
  self~assertTrue(files.~hasItem(thisFile~absolutePath), "SysFileTree should find" thisFile~absolutePath)

-- leading blanks in file name may be stripped off
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
