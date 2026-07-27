/* extracted from CONSTANT::test_constant_method_properties */
::routine main public
  m = .test~method('S1')

  self~assertTrue(m~isConstant)
  self~assertFalse(m~isPrivate)
  self~assertFalse(m~isGuarded)
  self~assertFalse(m~isAttribute)
  self~assertFalse(m~isProtected)
  self~assertFalse(m~isAbstract)


/** The following are tests syntax errors with the constant directive.  Since
 *  putting incorrect syntax for the directive in this file would cause the test
 *  group to not run, the bad syntax tests are done by using the .Package class.
 *  A package is created with the code for the test.  The syntax errors are then
 *  raised during the instantiation of the Package object, when the code is
 *  parsed.
 */

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
