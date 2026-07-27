/* extracted from Directory::test_setMethod_unSetMethod */
::routine main public

  d=.directory~new

  prePostFix="RGF"
  idxVal="rgf"
  itemValue="Rony G. Flatscher"


  m1=.array~of( "use arg name                      ;" -
                "return 'RGF'name~strip~reverse'RGF'")

  d~setMethod("unknown", m1)

  d~setMethod("reverse", "return 'oha, dackel' ")


  d~rgf=itemValue
  self~assertEquals(itemValue, d~rgf)
  self~assertEquals(itemValue, d~entry(idxVal))
  self~assertEquals(itemValue, d~at(idxVal~translate))

  newIdxVal=" nixi "
  expectedVal=prePostFix || newIdxVal~strip~reverse~translate || prePostFix
  self~assertEquals(expectedVal, d~entry(newIdxVal))

  d~setMethod("unknown")      -- unset the method
  self~assertNull(d~entry(newIdxVal))

  testVal='oha, dackel'
  self~assertEquals(testVal, d~reverse)

  d~unSetMethod("REVERSE")
  self~assertNull(d~reverse)




/* Test for syntax errors. <---  <---  <---   */

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
