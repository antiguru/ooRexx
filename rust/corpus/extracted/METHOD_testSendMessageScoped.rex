/* extracted from METHOD::testSendMessageScoped */
::routine main public
  tester = .METHODTester~new
  o = .Scoped3~new
  self~assertSame("S3 1 2 3",  tester~TestSendMessageScoped(o, "STRING", .Scoped3, .Array~of(1, 2, 3)))
  self~assertSame("S2",        tester~TestSendMessageScoped(o, "STRING", .Scoped2, .Array~new(0)))
  self~assertSame("S1",        tester~TestSendMessageScoped(o, "STRING", .Scoped1, .Array~new))
  self~assertSame("a SCOPED3", tester~TestSendMessageScoped(o, "STRING", .Object,  .Array~new(0)))


-- test class for SendMessageScoped
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
