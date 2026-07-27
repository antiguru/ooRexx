/* extracted from rxQueue::test_rxqueue_maximum_line_length */
::routine main public

  -- rexref says, 65472 chars allowed
  a = .Array~new(2)~fill("x"~copies(65472))
  long = .TemporaryTestFile~new(self, "rxqueue_maximum_line_length")
  long~create(a)
  .stdque~empty

  -- test piping stdin
  select case .ooRexxUnit.OSName
    when "WINDOWS" then
      "type" long~quotedName "| rxqueue"
    otherwise
      "cat" long~quotedName "| rxqueue"
  end
  self~assertEquals(2, queued())
  self~assertEquals(a~makeString("c"), .stdque~makeArray~makeString("c"))
  self~assertEquals(0, queued())

  -- test redirecting stdin
  "rxqueue <" long~quotedName
  self~assertEquals(2, queued())
  self~assertEquals(a~makeString("c"), .stdque~makeArray~makeString("c"))
  self~assertEquals(0, queued())

  long~delete


-- test for [bugs:#1471] RXQUEUE loses lineends every 4096 chars
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
