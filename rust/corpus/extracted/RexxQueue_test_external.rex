/* extracted from RexxQueue::test_external */
::routine main public
  name = "test_external"
  .RexxQueue~delete(name) -- just to make sure
  self~assertFalse(.RexxQueue~exists(name))

  -- create the named queue in a different Rexx interpreter instance
  call execRexxInline ".RexxQueue~new('" || name || "')"
  self~assertTrue(.RexxQueue~exists(name~upper))
  q = .RexxQueue~new(name)
  q~say("say")

  -- queue an item in a different Rexx interpreter instance
  call execRexxInline ".RexxQueue~new('" || name || "')~queue('external')"
  self~assertSame(2, q~queued)
  self~assertSame("say", q~pull)
  self~assertSame("external", q~pull)

  -- queue an item with the RXQUEUE filter
  address "" "echo RXQUEUE| rxqueue" name "/LIFO"
  self~assertSame(1, q~queued)
  self~assertSame("RXQUEUE", q~pull)

  -- delete queue in a different Rexx interpreter instance
  call execRexxInline ".RexxQueue~new('" || name || "')~delete"
  self~assertFalse(.RexxQueue~exists(name~upper))

-- external queue used by PUSH/QUEUE and PULL/PARSE PULL instructions
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
