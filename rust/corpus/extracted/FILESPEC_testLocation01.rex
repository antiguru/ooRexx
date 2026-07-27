/* extracted from FILESPEC::testLocation01 */
::routine main public
  if .ooRexxUnit.OSName == "WINDOWS" then do
      self~assertSame("", filespec("L", "Foo.bar"))
      self~assertSame("a\b\", filespec("L", "a\b\Foo.bar"))
      self~assertSame("\a\b\", filespec("l", "\a\b\Foo.bar"))
      self~assertSame("c:\a\b\", filespec("L", "c:\a\b\Foo.bar"))
      self~assertSame("\a\b\", filespec("l", "\a\b\Foo Bar.dat"))
      self~assertSame("\a\b c\", filespec("l", "\a\b c\Foo.bar"))
      self~assertSame("", filespec("l", ""))
  end
  else do
      self~assertSame("", filespec("L", "Foo.bar"))
      self~assertSame("a/b/", filespec("l", "a/b/Foo.bar"))
      self~assertSame("/a/b/", filespec("L", "/a/b/Foo.bar"))
      self~assertSame("", filespec("l", ""))
  end

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
