/* extracted from SysFileTree::test_arg_two */
::routine main public
  self~assertSame(0, SysFileTree("%", stem.))
  self~assertSame(0, stem.0)

  stem.0 = "none"
  self~assertSame(0, SysFileTree("%", stem.))
  self~assertSame(0, stem.0)

  self~assertSame(0, SysFileTree("%", "stem."))
  self~assertSame(0, stem.0)

  name = "st"
  self~assertSame(0, SysFileTree("%", name))
  self~assertSame(0, st.0)

  self~assertSame(0, SysFileTree("%", "name"))
  self~assertSame(0, name.0)

  s = .Stem~new
  self~assertSame(0, SysFileTree("%", s))
  self~assertSame(0, s[0])

  -- instead of a Stem, SysFileTree can also return an Array
  array = .Array~of(1, 2, 3)
  self~assertSame(0, SysFileTree("%", array))
  self~assertSame(0, array~items)


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
