/* extracted from directory::test_19 */
::routine main public
  dir = .directory~new
  dir~setMethod("Self", "return self")
  dir~setMethod("Super", "return super")

  self~assertSame(.directory, dir~super)
  self~assertSame(dir, dir~self)
  self~assertTrue(dir~hasIndex('SUPER'))
  self~assertTrue(dir~hasEntry('super'))

  dir~foo = "bar"

  self~assertEquals(3, dir~items)

  indexes = dir~allIndexes
  self~assertTrue(indexes~equivalent(.array~of("SELF", "SUPER", "FOO")))

  items = dir~allItems
  self~assertTrue(items~equivalent(.array~of(dir, .directory, "bar")))

  dir~self = "xyz"
  self~assertEquals("xyz", dir~self)

  self~assertEquals('xyz', dir~remove('SELF'))
  self~assertSame(.nil, dir~self)
  self~assertFalse(dir~hasIndex('SELF'))

  dir~unsetmethod('super')
  self~assertSame(.nil, dir~super)
  self~assertFalse(dir~hasEntry('super'))

  dir~setmethod('test', 'return "foo"')
  self~assertEquals('foo', dir~test)
  dir~setmethod('test')
  self~assertSame(.nil, dir~test)

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
