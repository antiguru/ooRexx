/* extracted from FUNCTION::test_global_setting */
::routine main public

  -- The base mechanisms have been tested above, here we're going to test
  -- that the global settings are used with commands and can be overridden on a base-by-case
  -- basis.

  signal off novalue

  address io with input using "This is a test" output stem a. error stem b.
  -- this will use the redirects
  address io 'INPUTOUTPUT'
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)
  self~assertSame(0, b.0)

  drop a. b.
  -- a command issued to this environment without using ADDRESS will also pick this up
  'INPUTERROR'

  self~assertSame(1, b.0)
  self~assertSame("This is a test", b.1)
  self~assertSame(0, a.0)

  drop a. b.

  -- override the INPUT source...this will use the global output ane error settings
  address io 'INPUTOUTPUT' with input using 'This is another test'
  self~assertSame(1, a.0)
  self~assertSame("This is another test", a.1)
  self~assertSame(0, b.0)

  drop a. b.

  -- now override OUTPUT and ERROR similarly
  address io 'INPUTOUTPUT' with output stem c.
  self~assertSame(1, c.0)
  self~assertSame("This is a test", c.1)
  self~assertSame(0, b.0)
  self~assertSame('A.0', a.0)      -- should be unchanged

  drop a. b. c.

  address io 'INPUTOUTPUT' with output stem c.
  self~assertSame(1, c.0)
  self~assertSame("This is a test", c.1)
  self~assertSame('A.0', a.0)
  self~assertSame(0, b.0)      -- should be unchanged

  drop a. b. c.

  -- now test NORMAL overrides

  address io 'INPUTOUTPUT' with input NORMAL
  -- this has nothing to read, but the output and error stems should reflect nothing written
  self~assertSame(0, a.0)
  self~assertSame(0, b.0)

  drop a. b.

  address io 'INPUTOUTPUT' with output NORMAL
  -- a. should be unchanged
  self~assertSame('A.0', a.0)
  self~assertSame(0, b.0)

  drop a. b.

  address io 'INPUTOUTPUT' with error NORMAL
  -- b. should be unchanged
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)
  self~assertSame('B.0', b.0)

  -- test the address toggle maintains the settings
  address command
  address

  drop a. b.

  -- this will use the redirects
  'INPUTOUTPUT'
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)
  self~assertSame(0, b.0)

  -- explicit switch back by name...still keeps the settings
  address command
  address io

  drop a. b.

  'INPUTOUTPUT'
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)
  self~assertSame(0, b.0)

  -- the address toggle only remembers two settings, but the override
  -- information should still be maintained as long as the context
  -- remains active.
  address xedit
  address command
  address io

  drop a. b.

  'INPUTOUTPUT'
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)
  self~assertSame(0, b.0)

  drop a. b.
  -- test that settings are saved on internal call return

  self~assertSame('A.0', a.0)
  self~assertSame('B.0', b.0)

  -- this should have been restored to the original settings
  'INPUTOUTPUT'
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)
  self~assertSame(0, b.0)

  return

  subroutine:


  address io with input using "This is another test" output stem c. error stem d.
  -- this will use the new redirects
  'INPUTOUTPUT'
  self~assertSame(1, c.0)
  self~assertSame("This is another test", c.1)
  self~assertSame(0, d.0)
  return

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
