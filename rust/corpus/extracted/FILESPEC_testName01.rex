/* extracted from FILESPEC::testName01 */
::routine main public
  if .ooRexxUnit.OSName == "WINDOWS" then do
      self~assertSame("Foobar", filespec("n", "Foobar"))
      self~assertSame("Foo.bar", filespec("n", "Foo.bar"))
      self~assertSame("Foo.bar", filespec("N", "a\b\Foo.bar"))
      self~assertSame("Foo.bar", filespec("n", "a\b.c\Foo.bar"))
      self~assertSame("Foo Bar.bar", filespec("n", "a\b.c\Foo Bar.bar"))
      self~assertSame("Foo Bar", filespec("n", "a\b.c\Foo Bar"))
      self~assertSame("", filespec("n", "a\b.c\"))
      self~assertSame("Foo.bar", filespec("n", "..\b\Foo.bar"))
      self~assertSame("", filespec("n", "..\b\"))
      self~assertSame("", filespec("n", "c:"))
      self~assertSame("Foo.bar", filespec("n", "c:Foo.bar"))
      self~assertSame("", filespec("n", ""))
  end
  else do
      self~assertSame("Foobar", filespec("n", "Foobar"))
      self~assertSame("Foo.bar", filespec("n", "Foo.bar"))
      self~assertSame("Foo.bar", filespec("N", "a/b/Foo.bar"))
      self~assertSame("Foo.bar", filespec("n", "a/b.c/Foo.bar"))
      self~assertSame("", filespec("n", "a/b.c/"))
      self~assertSame("Foo.bar", filespec("n", "../b/Foo.bar"))
      self~assertSame("", filespec("n", "../b/"))
      self~assertSame("", filespec("n", ""))
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
