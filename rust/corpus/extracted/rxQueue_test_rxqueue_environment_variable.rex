/* extracted from rxQueue::test_rxqueue_environment_variable */
::routine main public
  self~assertSame("SESSION", .stdque~get)
  .stdque~empty
  self~assertSame(0, .stdque~queued)
  q = .RexxQueue~new("rxqueue_environment_variable")
  q~empty
  self~assertSame(0, .stdque~queued)

  file = .TemporaryTestFile~new(self, "rxqueue_environment_variable")
  file~create(("line 1", "line 2", "line 3"))

  -- RXQUEUE environment variable not set, default should be SESSION
  "rxqueue <" file~quotedName
  self~assertSame(3, .stdque~queued)
  self~assertSame(0, q~queued)
  self~assertSame(3, .stdque~queued)

  -- RXQUEUE environment variable set
  call value "RXQUEUE", q~get, "ENVIRONMENT"
  "rxqueue <" file~quotedName
  call value "RXQUEUE", .nil, "ENVIRONMENT"
  self~assertSame(3, .stdque~queued)
  self~assertSame(3, q~queued)

  -- RXQUEUE environment variable set, but queue name argument specified
  call value "RXQUEUE", q~get, "ENVIRONMENT"
  "rxqueue session /fifo <" file~quotedName
  call value "RXQUEUE", .nil, "ENVIRONMENT"
  self~assertSame(6, .stdque~queued)
  self~assertSame(3, q~queued)

  -- now again with piped input
  cat = (.ooRexxUnit.OSName == "WINDOWS")~?("TYPE", "cat")
  .stdque~empty
  q~empty

  -- RXQUEUE environment variable not set, default should be SESSION
  cat file~quotedName "| rxqueue"
  self~assertSame(3, .stdque~queued)
  self~assertSame(0, q~queued)

  -- RXQUEUE environment variable set
  call value "RXQUEUE", q~get, "ENVIRONMENT"
  cat file~quotedName "| rxqueue"
  call value "RXQUEUE", .nil, "ENVIRONMENT"
  self~assertSame(3, .stdque~queued)
  self~assertSame(3, q~queued)

  -- RXQUEUE environment variable set, but queue name argument specified
  call value "RXQUEUE", q~get, "ENVIRONMENT"
  cat file~quotedName "| rxqueue session"
  call value "RXQUEUE", .nil, "ENVIRONMENT"
  self~assertSame(6, .stdque~queued)
  self~assertSame(3, q~queued)


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
