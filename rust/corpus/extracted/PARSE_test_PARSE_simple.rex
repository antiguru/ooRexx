/* extracted from PARSE::test_PARSE_simple */
::routine main public

   parse value 'time and tide' with var1 var2 var3
   self~assertSame('time', var1)
   self~assertSame('and', var2)
   self~assertSame('tide', var3)


   /* PARSE VALUE using a variable as the source string to parse    */
   string='time and tide'
   parse value string with var1 var2 var3           /* same results */
   self~assertSame('time', var1)
   self~assertSame('and', var2)
   self~assertSame('tide', var3)


   /* PARSE VAR example                                             */
   stars='Sirius Polaris Rigil'
   parse var stars star1 star2 star3
   self~assertSame('Sirius', star1)
   self~assertSame('Polaris', star2)
   self~assertSame('Rigil', star3)


   /* More variables in template than (words in) the source string  */
   satellite='moon'
   parse var satellite Earth Mercury               /* Earth='moon'  */
   self~assertSame('', Mercury)


   /* More (words in the) source string than variables in template  */
   satellites='moon Io Europa Callisto...'
   parse var satellites Earth Jupiter              /* Earth='moon'  */
   self~assertSame('Io Europa Callisto...', Jupiter)


   /* Preserving extra blanks                                       */
   solar5='Mercury Venus  Earth   Mars     Jupiter  '
   parse var solar5 var1 var2 var3 var4
   self~assertSame('Mercury', var1)
   self~assertSame('Venus', var2)
   self~assertSame('Earth', var3)
   self~assertSame('  Mars     Jupiter  ', var4)


   parse value '   Pluto   ' with var1
   self~assertSame('   Pluto   ', var1)


   /* Period as a placeholder                                       */
   stars='Arcturus Betelgeuse Sirius Rigil'
   parse var stars . . brightest .
   self~assertSame('Sirius', brightest)


   /* Alternative to period as placeholder                          */
   stars='Arcturus Betelgeuse Sirius Rigil'
   parse var stars drop junk brightest rest
   self~assertSame('Sirius', brightest)



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
