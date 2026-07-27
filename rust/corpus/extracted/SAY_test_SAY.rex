/* extracted from SAY::test_SAY */
::routine main public
   .output~destination(.myLogger~new)     -- let the SAY messages be sent to the local logger class
   SAY                        -- outputs empty string
   self~assertEquals("", .output~current~pull)

   SAY ""                     -- output empty string
   self~assertEquals("", .output~current~pull)

   a="   "                    -- output blank string
   say a
   tmp=.output~current~pull
   self~assertEquals("", tmp)
   self~assertNotSame("", tmp)
   self~assertSame(a, tmp)

   a=" anton was here, berta too...  " -- output non-blank string
   say a
   tmp=.output~current~pull
   self~assertEquals(a, tmp)
   self~assertSame(a, tmp)

   a=xrange("00"x, "ff"x)             -- output all characters, including control characters
   say a
   tmp=.output~current~pull
   self~assertEquals(a, tmp)
   self~assertSame(a, tmp)

-- test for [bugs:#1544] SAY raises NOTREADY
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
