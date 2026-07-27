/* extracted from socketClass::test_options_boolean */
::routine main public
  s = .Socket~new
  -- on Linux setting SO_DEBUG to true needs CAP_NET_ADMIN Capabilities,
  -- else fails with Permission denied - so we don't test it here
  -- on OpenBSD, setting SO_DONTROUTE returns .nil, so don't test it
  do opt over "keepalive", "oobinline", "reuseaddr"
    option = "so_"opt
    -- default is .false for all boolean options
    self~assertFalse(s~getOption(option), option~upper "default should be .false")
    -- set option to true
    self~assertSame(0, s~setOption(option, .true), option~upper)
    -- same as above: may be .true or 2^n on some platforms
    value = s~getOption(option)
    self~assertTrue(value > 0, option~upper "returns" value)
    -- set back to .false
    self~assertSame(0, s~setOption(option, .false), option~upper)
    self~assertFalse(s~getOption(option), option~upper "should be .false")
  end
  self~assertSame(0, s~close)

-- SO_LINGER expects two numbers (but socket.cls currently doesn't check)
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
