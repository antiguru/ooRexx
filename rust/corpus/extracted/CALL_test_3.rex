/* extracted from CALL::test_3 */
::routine main public
   -- Create our external functions.
   path = .ooRexxUnit.dir || .ooRexxUnit.directory.separator
   src = .array~new()
   src[1] = "Parse Source fn"
   src[2] = "   num=Arg()"
   src[3] = "   sum=0"
   src[4] = "   Do i=1 To num"
   src[5] = "     sum=sum+Arg(i)"
   src[6] = "     End"
   src[7] = "   Return sum"
   call createFile src, path'SCICAL3A'

   -- now do the tests
   ai=0
   Call a a(0),a(1),a(2),a(3),a(4),a(5),a(6),a(7),a(8),a(9)
   self~assertSame(result, xrange('0','9'))

   ai=0
   Call a a(0),a(1),a(2),a(3),a(4),a(5),a(6),a(7),a(8),a(9),10
   self~assertSame(result, '012345678910')

   v202='vVv'
   v203="COPIES"('xx',5000)
   v207.='ab'
   v207.1='cd'
   Call sub0201 'abc'; self~assertSame(result, 201)
   Call sub0202 v202;  self~assertSame(result, 0202)
   Call sub0203 v203;  self~assertSame(result, 203)
   Call sub0204 (2+3); self~assertSame(result, 204)
   Numeric Digits 2
   Call sub0205 +205;  self~assertSame('2.1E+2', +result)
   Numeric Digits 9
   Call sub0206 12a,'0102'X;
                       self~assertSame(result, '0201'x)
   Call sub0207 v207.,v207.1
                       self~assertSame(result, 'abcd')
   Call sub0208 '',''; self~assertSame(result, '')
   Call sub0209 bitor('0102'X,'2011'X),'a,' 'b,' 'and c'
                       self~assertSame(result, 'a,*b,*and*c')
   Do i=1 to 21
     a.i=i
     End
   Call a20 a.1, a.2, a.3, a.4, a.5, a.6, a.7, a.8, a.9, a.10,,
            a.11,a.12,a.13,a.14,a.15,a.16,a.17,a.18,a.19,a.20
   self~assertSame(result, 210)
   Call scical3a a.1, a.2, a.3, a.4, a.5, a.6, a.7, a.8, a.9, a.10,,
                 a.11,a.12,a.13,a.14,a.15,a.16,a.17,a.18,a.19,a.20
   self~assertSame(result, 210)
   Call a20 a.1, a.2, a.3, a.4, a.5, a.6, a.7, a.8, a.9, a.10,,
            a.11,a.12,a.13,a.14,a.15,a.16,a.17,a.18,a.19,a.20,a.21
   Call scical3a a.1, a.2, a.3, a.4, a.5, a.6, a.7, a.8, a.9, a.10,,
                 a.11,a.12,a.13,a.14,a.15,a.16,a.17,a.18,a.19,a.20,a.21

   -- now remove the external functions
   call deleteFile path'SCICAL3A'
   return

   a:
   If "ARG"()>1 Then Do;               /* concatenate all arguments      */
      s='';
      Do ai=1 To "ARG"();
         self~assertSame((ai-1), "ARG"(ai))
         s=s||"ARG"(ai);
         End;
      Return s
      End
                                       /* call as argument expression    */
   self~assertSame("ARG"(1), ai)
   ai=ai+1;                            /* increment to new number        */
   Return ai-1                         /* return old number as value     */

   sub0201: self~assertSame(arg(1), 'abc'); Return '201'
   sub0202: self~assertSame(arg(1), 'vVv'); Return 0202
   sub0203: self~assertSame(arg(1), "COPIES"('x',10000)); Return +0.0203e+004
   sub0204: self~assertSame(arg(1), ""5); Return 200+4
   sub0205: Numeric Digits 9
            self~assertSame(arg(1), "2.1E+2"); Return 205
   sub0206: self~assertSame(arg(1), "12A")
            self~assertSame(arg(2), "0102"X)
            Return "RIGHT"("ARG"(2),1)"LEFT"("ARG"(2),1)
   sub0207: self~assertSame(arg(1), "ab")
            self~assertSame(arg(2), "cd")
            Return "ARG"(1)||"ARG"(2)
   sub0208: self~assertSame(arg(1), "")
            self~assertSame(arg(2), ""x)
            v208=''
            Return "ARG"(1)||"ARG"(2)||v208
   sub0209: self~assertSame(arg(1), "2113"X)
            self~assertSame(arg(2), 'a, b, and c')
            Return "TRANSLATE"("ARG"(2),'*','             ')
   a20: Procedure
        sum=0
        Do i=1 To arg()
           sum=sum+Arg(i)
           End
        Return sum

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
