/* extracted from PARSE::Test_654 */
::routine main public
   Call sub 010,020,030,040,050,060,070,080,090,100,,
            110,120,130,140,150,160,170,180,190,200,,
            210,220,230,240,250,260,270,280,290,300,,
            310,320,330,340,350,360,370,380,390,400
   self~assertSame(result, '010.400')
   return
   sub:
     Parse Arg a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,,
               b1,b2,b3,b4,b5,b6,b7,b8,b9,b10,,
               c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,,
               d1,d2,d3,d4,d5,d6,d7,d8,d9,d10
   Return a1||'.'||d10

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
