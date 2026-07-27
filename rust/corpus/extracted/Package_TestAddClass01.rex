/* extracted from Package::TestAddClass01 */
::routine main public
  src = .array~new()
  src[1] = "::class class1"

  package = .package~new("ADDCLASSTEST1", src)

  class = .object~subclass("TEST2")

  package~addClass("TEST2", class)

  classes = package~classes
  self~assertSame(2, classes~items)
  self~assertSame(class, classes["TEST2"])
  self~assertSame(0, package~publicClasses~items)
  self~assertSame(class, package~findClass("TEST2"))
  -- should not be found as a public class
  self~assertSame(.nil, package~findPublicClass("TEST2"))
  -- test that find also locates the REXX-defined classes
  self~assertSame(.array, package~findClass("ARRAY"))
  self~assertSame(.array, package~findPublicClass("ARRAY"))

  -- now add a public class
  class = .object~subclass("TEST3")
  package~addPublicClass("TEST3", class)

  self~assertSame(class, package~findClass("TEST3"))
  -- should be found as a public class
  self~assertSame(class, package~findPublicClass("TEST3"))


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
