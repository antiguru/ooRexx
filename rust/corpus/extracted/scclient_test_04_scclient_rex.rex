/* extracted from scclient::test_04_scclient.rex */
::routine main public

  Line.1  = 'Connect failed: ECONNREFUSED'
  Line.0  = 1

-- assuming the server is not running Connect failed will be the result

  inArr=('Hello World')

  prgPathName = locateSamplePrg("scclient.rex")

  SELECT CASE .ooRexxUnit.OSName
    when 'WINDOWS' then address command 'rexx' '"'prgPathName'"' with input using (inArr) output stem myStem.
    otherwise           address ''      'rexx' prgPathName with input using (inArr) output stem myStem.
  END /* Select */

  DO i= 1 TO myStem.0
    self~assertEquals(Line.i,myStem.i)
  END i

  use arg prefix = "sc"
--use arg prefix = "sf"

-- now make a proper test of the client-server communication;
-- example code provided by Erich Steinböck, TNX!
-- passing result with help of Rony G. Flatscher, TNX!

  LineS.1 = 'Server listening at 0.0.0.0:50010'		--macOS
  --LineS.1 = 'Server listening at 192.168.1.10:50010'	--Win
  LineS.2 = 'Press [Enter] To Shutdown'
  --<3>
  LineS.3 = 'shutdown in 3 sec'
  LineS.0 = 3

  LineC.1 = 'type "X" to exit'
  LineC.2 = 'Send To Server: Server responded: Echo: 111'
  LineC.3 = 'Send To Server: Server responded: Echo: 222'
  LineC.4 = 'Send To Server: Server responded: Echo: 333'
  LineC.5 = 'Send To Server:'                              --"x" swallowed by client
  LineC.0 = 5

  LineC2.1 = 'type "X" to exit'
  LineC2.2 = 'Send To Server: Server responded: Echo: 444'
  LineC2.3 = 'Send To Server: Server responded: Echo: 555'
  LineC2.4 = 'Send To Server: Server responded: Echo: 666'
  LineC2.5 = 'Send To Server:'                              --"x" swallowed by client
  LineC2.0 = 5

  serverInArray      = (3)
  serveryOutputStem. = .stem~new

  clientInArray      = ('111','222','333', 'x')
  clientOutputStem.  = .stem~new

  clientInArray2      = ('444','555','666', 'x')
  clientOutputStem2.  = .stem~new

  currPath   = directory()
  samplePath = filespec('location',prgPathName)
-- on Windows we need the drive

  -- We need to move to where the sample is
  SELECT CASE .ooRexxUnit.OSName
    when 'WINDOWS' then address command 'cd' '"'samplePath'"'
    otherwise           address ''      'cd' samplePath
  END /* Select */

-- create an instance of the sc or sf client-Server test class
  job = .Job~new(prefix)

-- start the server
  job~start("server", serverInArray, serveryOutputStem.)

  call SysSleep 0.5 -- wait for server to become ready

-- start one client
  job~start("client", clientInArray, clientOutputStem.)

-- start a 2nd client
  job~start("client", clientInArray2, clientOutputStem2.)

  call SysSleep 5 -- wait to be sure the server is closed before evaluating results

  DO i= 1 TO LineS.0
    if i= 1 then self~assertEquals(LineS.i~subWord(1,3),serveryOutputStem.i~subWord(1,3))
    else         self~assertEquals(LineS.i,serveryOutputStem.i)
  END i

  DO i= 1 TO LineC.0
    self~assertEquals(LineC.i, clientOutputStem.i)
  END i

  DO i= 1 TO LineC2.0
    self~assertEquals(LineC2.i, clientOutputStem2.i)
  END i

  -- Then move back to where we were before
  SELECT CASE .ooRexxUnit.OSName
    when 'WINDOWS' then address command 'cd' '"'currPath'"'
    otherwise           address ''      'cd' currPath
  END /* Select */

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
