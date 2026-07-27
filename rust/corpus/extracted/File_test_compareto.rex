/* extracted from File::test_compareto */
::routine main public
  root = .File~listRoots[1]
  case = .File~isCaseSensitive
  aLower = .File~new("a", root)
  aUpper = .File~new("A", root)
  bUpper = .File~new("B", root)
  one = .File~new(1, root)
  self~assertSame(0, aLower~compareTo(aLower))
  self~assertTrue(aLower = aLower)
  self~assertSame(case~?(1, 0), aLower~compareTo(aUpper))
  self~assertSame(case~?(.false, .true), aLower < bUpper)
  self~assertSame(case~?(1, -1), aLower~compareTo(bUpper))
  self~assertSame(-1, one~compareTo(aLower))
  self~assertTrue(one < aLower)
  sorted = ""
  do file over (aLower, bUpper, one)~sort
    sorted = sorted file~name
  end
  self~assertSame(case~?("1 B a", "1 a B"), sorted~strip)


-- absolutePath/absoluteFile tests

-- too many arguments
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
