/* extracted from CALL::test_7 */
::routine main public
   i=0
   Call a 'a'; self~assertSame((result i), 'a 1')
   Call b 'b'; self~assertSame((result i), 'b 2')
   Call c 'c'; self~assertSame((result i), 'c 3')
   Call d 'd'; self~assertSame((result i), 'd 4')
   Call e 'e'; self~assertSame((result i), 'e 5')
   Call f 'f'; self~assertSame((result i), 'f 6')
   Call g 'g'; self~assertSame((result i), 'g 6')
   Call h 'h'; self~assertSame((result i), 'h 6')
   Call i 'i'; self~assertSame((result i), 'i 6')
   Call k 'k'; self~assertSame((result i), 'k 7')

   i=0
   self~assertSame((a('a') i b('b') i c('c') i), 'a 1 b 2 c 3')
   self~assertSame((d('d') i), 'd 4')
   self~assertSame((e('e') i), 'e 5')
   self~assertSame((f('f') i), 'f 6')
   self~assertSame((g('g') i), 'g 6')
   self~assertSame((h('h') i), 'h 6')
   self~assertSame((i('i') i), 'i 6')
   self~assertSame((k('k') i), 'k 7')
   return

   a: b:
   c:
     i=i+1
     Return("ARG"(1))

   d: e:
   f: Procedure Expose j i j k
     i=i+1
     Return("ARG"(1))

   g: h:
   i: Procedure
     i=1
     Return("ARG"(1))

   k: Procedure Expose i
     i=i+1
     Return("ARG"(1))

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
