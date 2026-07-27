/* extracted from CALL::test_4 */
::routine main public
   -- Create our external functions.
   path = .ooRexxUnit.dir || .ooRexxUnit.directory.separator
   src = .array~new()
   src[1] = "Parse Version version"
   src[2] = "Parse Source fn"
   src[3] = "  Parse Version sys rexxlevel ."
   src[4] = "  Address CMD"
   src[5] = "  Numeric Fuzz 0"
   src[6] = "  Numeric Digits 100"
   src[7] = "  Numeric Form Scientific"
   src[8] = "  t='TIME'('R')"
   src[9] = "  dff=digits() fuzz() form()"
   src[10] = "  Return dff"
   call createFile src, path'SCICAL4A'

   -- now do the tests
   cmd = address()
   cnt.=0
   g_.=''
   Numeric Digits 5
   Numeric Fuzz 1
   Numeric Form Engineering
   call time 'r'
   Call sysSLEEP 1.1   -- a little longer than we test for to account for clock resolution problems.

   Do i=1 to 2
      ic=0                              /* Iteration counter              */
      Do j=1 to 5
         Do k=1 to 3
           If j=2 & k=2 Then Do
              If i=1 Then
               Call isub
              Else
               Call SCICAL4a
              subdff=result
              dff=digits() fuzz() form()
              self~assertSame(dff, '5 1 ENGINEERING')
              self~assertSame(subdff, '100 0 SCIENTIFIC')
              self~assertSame(novalued_var, 'NOVALUED_VAR')
              self~assertSame(novalued_var, 'NOVALUED_VAR')
              id=1001; Call gt "TIME"('E'),1
              self~assertSame("ADDRESS"(), cmd)
              End
           ic=ic+1
           End
        End
     self~assertSame(ic, 15)
     End
   self~assertSame(4, cnt.ok, "Okay count should be 4")

   -- now remove the external functions
   call deleteFile path'SCICAL4A'
   return

   isub:
   /***********************************************************************
   * Internal subroutine that does all kinds of changes to the
   * "important settings"
   ***********************************************************************/
     Numeric Fuzz 0
     Numeric Digits 100
     Numeric Form Scientific
     Signal On Novalue
     Signal Off Syntax
     id=1004; Call gt "TIME"('E'),1
     tt="TIME"('R')
     id=1005; Call lt "TIME"('E'),1
     dff=digits() fuzz() form()
     Signal subret
     garbage in sub
     the signal within the subroutine has no effect on the do control

   subret:
      Address ''
      self~assertSame("ADDRESS"(), '')
      Return dff
   gt: Procedure Expose id cnt. self
      self~assertTrue(arg(1)>=arg(2), "subTest4-gt id:" id "TIME('E')>1")
      cnt.ok=cnt.ok+1
      Return
   lt: Procedure Expose id cnt. self
      self~assertTrue(arg(1)<arg(2), "subTest4-lt id:" id "TIME('E')>1")
      cnt.ok=cnt.ok+1
      Return

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
