/* extracted from ASSIGNMENT::test_6 */
::routine main public
   y=0
   x=y
   a=17
   Address nowhere
   /*1*/a/*2*/=/*3*/2/*4*/;
   ,

   b=3
   c,
   =4
   d=,
   5
   e=6,

   self~assertSame((a b c d e), (2 3 4 5 6))
   e=7
   address     =2
   arg         =4
   call        =6
   do          =8
   drop        =10
   exit        =12
   if          =14
   interpret   =16
   iterate     =18
   leave       =20
   nop         =22
   numeric     =24
   options     =26
   parse       =28
   procedure   =30
   pull        =32
   push        =34
   queue       =36
   return      =38
   say         =40
   select      =42
   signal      =44
   trace       =46
   upper       =48
   self~assertSame(address, 2)
   self~assertSame(arg, 4)
   self~assertSame(call, 6)
   self~assertSame(do, 8)
   self~assertSame(drop, 10)
   self~assertSame(exit, 12)
   self~assertSame(if, 14)
   self~assertSame(interpret, 16)
   self~assertSame(iterate, 18)
   self~assertSame(leave, 20)
   self~assertSame(nop, 22)
   self~assertSame(numeric, 24)
   self~assertSame(options, 26)
   self~assertSame(parse, 28)
   self~assertSame(procedure, 30)
   self~assertSame(pull, 32)
   self~assertSame(push, 34)
   self~assertSame(queue, 36)
   self~assertSame(return, 38)
   self~assertSame(say, 40)
   self~assertSame(select, 42)
   self~assertSame(signal, 44)
   self~assertSame(trace, 46)
   self~assertSame(upper, 48)

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
