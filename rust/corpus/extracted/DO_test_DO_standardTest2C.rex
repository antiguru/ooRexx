/* extracted from DO::test_DO_standardTest2C */
::routine main public
   -- Create our external functions.
   path = .ooRexxUnit.dir || .ooRexxUnit.directory.separator
   src = .array~new()
   src[1] = "Parse arg ii bb tt ff ."
   src[2] = "extloop:"
   src[3] = "c=0"
   src[4] = "Do k=ii for ff by bb to tt"
   src[5] = "c=c+1"
   src[6] = "End"
   src[7] = "Return c"
   call createFile src, path'SCIDOS2A'

   -- now do the tests
   expri.1=6; expri.2=3; expri.3=0; expri.4=-3; expri.5=-6
   exprb.1=4; exprb.2=3; exprb.3=-3;exprb.4=-4
   exprt.1=10;exprt.2=5; exprt.3=0; exprt.4=-5; exprt.5=-10;
   exprf.1=15; exprf.2=7 ; exprf.3=0 ;
   is.=0
   Do i=1 to 5;
      Do b=1 to 4;
         Do t=1 to 5;
            Do f=1 to 3;
               Do k=expri.i For exprf.f By exprb.b To exprt.t
               is.i.b.t.f=(is.i.b.t.f)+1
               End k
            End f
         End t
      End b
   End i
   Do i=1 to 5;
      Do b=1 to 4;
         Do t=1 to 5;
            Do f=1 to 3;
            self~AssertSame(is.i.b.t.f, "SCIDOS2A"(expri.i exprb.b exprt.t exprf.f))
            End f
         End t
      End b
   End i

   -- now remove the external functions
   call deleteFile path'SCIDOS2A'

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
