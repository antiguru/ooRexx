/* extracted from DATE::test_DATE */
::routine main public
    tmpD='13 Nov 1996'

    self~assertSame('13 Nov 1996', DATE(   , tmpD ))
    self~assertSame(728975, DATE('B', tmpD ))             -- error: 728609             )
    self~assertSame(318, DATE('D', tmpD ))             -- error: 317                )
    self~assertSame('13/11/96', DATE('E', tmpD ))
    self~assertSame('13 November 1996', DATE('L', tmpD ))
    self~assertSame('November', DATE('M', tmpD ))
    self~assertSame('13 Nov 1996', DATE('N', tmpD ))
    self~assertSame('96/11/13', DATE('O', tmpD ))
    self~assertSame('19961113', DATE('S', tmpD ))
    self~assertSame('11/13/96', DATE('U', tmpD ))
    self~assertSame('Wednesday', DATE('W', tmpD ))             -- error: 'Monday'           )

    self~assertSame('23/02/13', DATE('O','13 Feb 1923'))
    self~assertSame('50/06/01', DATE('O','06/01/50','U'))

    self~assertSame('1996-02-13', DATE('S','13 Feb 1996','N','-'))
    self~assertSame('13Feb1996', DATE('N','13 Feb 1996','N',""))
    self~assertSame('13-Feb-1996', DATE('N','13 Feb 1996','N','-'))
    self~assertSame('500601', DATE('O','06/01/50','U',""))
    self~assertSame('13.02.96', DATE('E','02/13/96','U','.'))
    self~assertSame('26_Mar_1998', DATE('N', '26 Mar 1998', ,'_'))

    -- self~assertSame(DATE('S','1996-11-13','S', ,"",'-'), '19961113') -- error: more than 5 arg() !
    self~assertSame('19961113', DATE('S','1996-11-13','S',"",'-'))

    self~assertSame('19961113', DATE('S','13-Nov-1996','N',"",'-'))
    self~assertSame('500601', DATE('O','06*01*50','U',"",'*'))
    self~assertSame('02/13/96', DATE('U','13.Feb.1996','N', ,'.'))


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
