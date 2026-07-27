/* extracted from FORWARD::test_forward_mixinclasses_01_class */
::routine main public
  o=.mixin2
  self~assertEquals("mixin2_class"  , o~clzInfo                        )
  self~assertEquals("mixin2_class"  , o~testForward_to_info_class      )
  self~assertEquals("base_class"    , o~testForwardSuper_Base_class    )
  self~assertEquals("mixin1a_class" , o~testForwardSuper_Mixin1a_class )
  self~assertEquals("mixin1b_class" , o~testForwardSuper_Mixin1b_class )
  self~assertEquals("mixin2_class"  , o~testForwardSuper_Mixin2_class  )

  self~expectSyntax(93.957)    -- 93.957 "Target object "a MIXIN2" is not a subclass of the message override scope (The NIXINOXI class)."
  self~assertEquals("nixinoxi", o~testForwardSuper_NixiNoxi_class)

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
