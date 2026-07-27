/* extracted from FILESPEC::testExtension01 */
::routine main public
  if .ooRexxUnit.OSName == "WINDOWS" then do
      self~assertSame("", filespec("E", "Foobar"))
      self~assertSame("bar", filespec("E", "Foo.bar"))
      self~assertSame("bar", filespec("E", "a\b\Foo.bar"))
      self~assertSame("bar", filespec("E", "a\b.c\Foo.bar"))
      self~assertSame("", filespec("E", "a\b.c\Foobar"))
      self~assertSame("", filespec("E", "a\b.c\Foo bar"))
      self~assertSame("bar", filespec("e", "..\b\Foo.bar"))
      self~assertSame("bar", filespec("e", "..\b\Foo Bar.bar"))
      self~assertSame("", filespec("e", "..\b\Foobar"))
      self~assertSame("bar", filespec("e", ".\b\Foo.bar"))
      self~assertSame("", filespec("e", ".\b\Foobar"))
      self~assertSame("", filespec("e", ""))
  end
  else do
      self~assertSame("", filespec("E", "Foobar"))
      self~assertSame("bar", filespec("E", "Foo.bar"))
      self~assertSame("bar", filespec("E", "a/b/Foo.bar"))
      self~assertSame("bar", filespec("E", "a/b.c/Foo.bar"))
      self~assertSame("", filespec("E", "a/b.c/Foobar"))
      self~assertSame("bar", filespec("e", "../b/Foo.bar"))
      self~assertSame("", filespec("e", "../b/Foobar"))
      self~assertSame("bar", filespec("e", "./b/Foo.bar"))
      self~assertSame("", filespec("e", "./b/Foobar"))
      self~assertSame("", filespec("e", ""))
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
