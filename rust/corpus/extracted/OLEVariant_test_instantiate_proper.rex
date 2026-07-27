/* extracted from OLEVariant::test_instantiate_proper */
::routine main public

    varVal = .array~new
    vt     = "VT_BOOL"
    flags  = "IN"

    ole = .oleVariant~new( varVal, vt, flags )
    a   = ole~!varValue_

    self~assertTrue((a~class == .array), "Variant value class should be array")
    self~assertSame(varVal, ole~!varValue_, "Variant value should be the value it was set as")
    self~assertEquals("VT_BOOL", ole~!varType_, "Variant type (string) should equal VT_BOOL")
    self~assertEquals("IN", ole~!paramFlags_, "Variant flags (string) should equal IN")
    self~assertTrue(ole~!clearVariant_, "Clear variant attribute must be true")

    -- Case insensitive should be allowed for variant type and param flags.
    varVal = "cat"
    vt     = "VT_bool"
    flags  = "IN,out"

    ole = .oleVariant~new( varVal, vt, flags )
    a   = ole~!varValue_

    self~assertTrue((a~class == .string), "Variant value class should be string")
    self~assertSame("cat", ole~!varValue_, "Variant value should be 'cat'")
    self~assertEquals("VT_bool", ole~!varType_, "Variant type (string) should equal VT_bool")
    self~assertEquals("IN,out", ole~!paramFlags_, "Variant flags (string) should equal IN,OUT")
    self~assertTrue(ole~!clearVariant_, "Clear variant attribute must be true")

    -- Spaces should not affect variant type or param flags.
    varVal = .list~of( "Sequoia", "Yosemite", "Rainier" )
    vt     = " VT_bool         "
    flags  = "IN,  out"

    ole = .oleVariant~new( varVal, vt, flags )
    a   = ole~!varValue_

    self~assertTrue((a~class == .list), "Variant value class should be list")
    self~assertSame("Sequoia", ole~!varValue_~at( 0 ), "Variant value object: list~at( 0 ) should be 'Sequoia'")
    self~assertSame("Yosemite", ole~!varValue_~at( 1 ), "Variant value object: list~at( 1 ) should be 'Yosemite'")
    self~assertSame("Rainier", ole~!varValue_~at( 2 ), "Variant value object: list~at( 2 ) should be 'Rainier'")
    self~assertEquals(vt, ole~!varType_, "Variant type (string) should equal" vt)
    self~assertEquals(flags, ole~!paramFlags_, "Variant flags (string) should equal" flags)
    self~assertTrue(ole~!clearVariant_, "Clear variant attribute must be true")


  -- End test_instantiateProper( )

  /* test_instantiate_errorXX( ) - - - - - - - - - - - - - - - - - - - - - - -*\

    The following methods all test .OLEVariant~new() using invalid (error
    producing) arguments.  Since a syntax error is raised, a large number of
    individual test cases need to be used rather than combining a number of
    assertions in one test case.

  \* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/

  -- No args is not allowed; must include the varValue argument.
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
