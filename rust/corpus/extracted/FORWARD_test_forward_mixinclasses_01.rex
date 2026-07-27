/* extracted from FORWARD::test_forward_mixinclasses_01 */
::routine main public
  o=.mixin2~new
  self~assertEquals("mixin2"  , o~info                     )
  self~assertEquals("mixin2"  , o~testForward_to_info      )
  self~assertEquals("base"    , o~testForwardSuper_Base    )
  self~assertEquals("mixin1a" , o~testForwardSuper_Mixin1a )
  self~assertEquals("mixin1b" , o~testForwardSuper_Mixin1b )
  self~assertEquals("mixin2"  , o~testForwardSuper_Mixin2  )

  self~expectSyntax(93.957)    -- 93.957 "Target object "a MIXIN2" is not a subclass of the message override scope (The NIXINOXI class)."
  self~assertEquals("nixinoxi", o~testForwardSuper_NixiNoxi)

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
