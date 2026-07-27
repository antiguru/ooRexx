/* extracted from sfserver::test_04_sfserver.rex */
::routine main public

  Line.1 = 'Server listening at 0.0.0.0:50010'		--macOS
  --Line.1 = 'Server listening at 192.168.1.10:50010'	--Win
  Line.2 = 'Press [Enter] To Shutdown'
  Line.0 = 2
  --<eol>

-- Start server and shut it down immediately

  inArr=(.endofline)

  prgPathName = locateSamplePrg("sfserver.rex")
 
  SELECT CASE .ooRexxUnit.OSName
    when 'WINDOWS' then address command 'rexx' '"'prgPathName'"' with input using (inArr) output stem myStem.
    otherwise           address ''      'rexx' prgPathName with input using (inArr) output stem myStem.
  END /* Select */

  DO i= 1 TO myStem.0	--different IP for different OS
    if i=1 then self~assertEquals(Line.i~subWord(1,3),myStem.i~subWord(1,3))
    else        self~assertEquals(Line.i,myStem.i)
  END i

-- Starting the server a 2nd time directly, should not provoke an error

  SELECT CASE .ooRexxUnit.OSName
    when 'WINDOWS' then address command 'rexx' '"'prgPathName'"' with input using (inArr) output stem myStem.
    otherwise           address ''      'rexx' prgPathName with input using (inArr) output stem myStem.
  END /* Select */

  DO i= 1 TO myStem.0	--different IP for different OS
    if i=1 then self~assertEquals(Line.i~subWord(1,3),myStem.i~subWord(1,3))
    else        self~assertEquals(Line.i,myStem.i)
  END i

-- Starting the server a 3rd time with delayed closure will give a different response

  Line.1 = 'Server listening at 0.0.0.0:50010'		--macOS
  --Line.1 = 'Server listening at 192.168.1.10:50010'	--Win
  Line.2 = 'Press [Enter] To Shutdown'
  --<3>
  Line.3 = 'shutdown in 3 sec'
  Line.0 = 3

  inArr=(3)

  SELECT CASE .ooRexxUnit.OSName
    when 'WINDOWS' then address command 'rexx' '"'prgPathName'"' with input using (inArr) output stem myStem.
    otherwise           address ''      'rexx' prgPathName with input using (inArr) output stem myStem.
  END /* Select */

  call SysSleep 5 -- wait to be sure the server is closed before evaluating results

  DO i= 1 TO myStem.0	--different IP for different OS
    if i=1 then self~assertEquals(Line.i~subWord(1,3),myStem.i~subWord(1,3))
    else        self~assertEquals(Line.i,myStem.i)
  END i

--we are done in this testgroup
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
