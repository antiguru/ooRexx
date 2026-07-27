/* extracted from ASSIGNMENT::test_5 */
::routine main public
   a=8; a.=8; a.8=88;
   b=4; b.=4; b.4=44;
   numeric digits 9
   id=1 ; i = 8 + 4  ;   self~assertSame(i, 12)
   id=2 ; i = 8 + b  ;   self~assertSame(i, 12)
   id=3 ; i = 8 + b.4  ;   self~assertSame(i, 52)
   id=4 ; i = 8 + b.  ;   self~assertSame(i, 12)
   id=5 ; i = 8 - 4  ;   self~assertSame(i, 4)
   id=6 ; i = 8 - b  ;   self~assertSame(i, 4)
   id=7 ; i = 8 - b.4  ;   self~assertSame(i, -36)
   id=8 ; i = 8 - b.  ;   self~assertSame(i, 4)
   id=9 ; i = 8 * 4  ;   self~assertSame(i, 32)
   id=10 ; i = 8 * b  ;   self~assertSame(i, 32)
   id=11 ; i = 8 * b.4  ;   self~assertSame(i, 352)
   id=12 ; i = 8 * b.  ;   self~assertSame(i, 32)
   id=13 ; i = 8 / 4  ;   self~assertSame(i, 2)
   id=14 ; i = 8 / b  ;   self~assertSame(i, 2)
   id=15 ; i = 8 / b.4  ;   self~assertSame(i, 0.181818182)
   id=16 ; i = 8 / b.  ;   self~assertSame(i, 2)
   id=17 ; i = 8 || 4  ;   self~assertSame(i, 84)
   id=18 ; i = 8 || b  ;   self~assertSame(i, 84)
   id=19 ; i = 8 || b.4  ;   self~assertSame(i, 844)
   id=20 ; i = 8 || b.  ;   self~assertSame(i, 84)
   id=21 ; i = 8 = 4  ;   self~assertSame(i, 0)
   id=22 ; i = 8 = b  ;   self~assertSame(i, 0)
   id=23 ; i = 8 = b.4  ;   self~assertSame(i, 0)
   id=24 ; i = 8 = b.  ;   self~assertSame(i, 0)
   id=25 ; i = a + 4  ;   self~assertSame(i, 12)
   id=26 ; i = a + b  ;   self~assertSame(i, 12)
   id=27 ; i = a + b.4  ;   self~assertSame(i, 52)
   id=28 ; i = a + b.  ;   self~assertSame(i, 12)
   id=29 ; i = a - 4  ;   self~assertSame(i, 4)
   id=30 ; i = a - b  ;   self~assertSame(i, 4)
   id=31 ; i = a - b.4  ;   self~assertSame(i, -36)
   id=32 ; i = a - b.  ;   self~assertSame(i, 4)
   id=33 ; i = a * 4  ;   self~assertSame(i, 32)
   id=34 ; i = a * b  ;   self~assertSame(i, 32)
   id=35 ; i = a * b.4  ;   self~assertSame(i, 352)
   id=36 ; i = a * b.  ;   self~assertSame(i, 32)
   id=37 ; i = a / 4  ;   self~assertSame(i, 2)
   id=38 ; i = a / b  ;   self~assertSame(i, 2)
   id=39 ; i = a / b.4  ;   self~assertSame(i, 0.181818182)
   id=40 ; i = a / b.  ;   self~assertSame(i, 2)
   id=41 ; i = a || 4  ;   self~assertSame(i, 84)
   id=42 ; i = a || b  ;   self~assertSame(i, 84)
   id=43 ; i = a || b.4  ;   self~assertSame(i, 844)
   id=44 ; i = a || b.  ;   self~assertSame(i, 84)
   id=45 ; i = a = 4  ;   self~assertSame(i, 0)
   id=46 ; i = a = b  ;   self~assertSame(i, 0)
   id=47 ; i = a = b.4  ;   self~assertSame(i, 0)
   id=48 ; i = a = b.  ;   self~assertSame(i, 0)
   id=49 ; i = a.8 + 4  ;   self~assertSame(i, 92)
   id=50 ; i = a.8 + b  ;   self~assertSame(i, 92)
   id=51 ; i = a.8 + b.4  ;   self~assertSame(i, 132)
   id=52 ; i = a.8 + b.  ;   self~assertSame(i, 92)
   id=53 ; i = a.8 - 4  ;   self~assertSame(i, 84)
   id=54 ; i = a.8 - b  ;   self~assertSame(i, 84)
   id=55 ; i = a.8 - b.4  ;   self~assertSame(i, 44)
   id=56 ; i = a.8 - b.  ;   self~assertSame(i, 84)
   id=57 ; i = a.8 * 4  ;   self~assertSame(i, 352)
   id=58 ; i = a.8 * b  ;   self~assertSame(i, 352)
   id=59 ; i = a.8 * b.4  ;   self~assertSame(i, 3872)
   id=60 ; i = a.8 * b.  ;   self~assertSame(i, 352)
   id=61 ; i = a.8 / 4  ;   self~assertSame(i, 22)
   id=62 ; i = a.8 / b  ;   self~assertSame(i, 22)
   id=63 ; i = a.8 / b.4  ;   self~assertSame(i, 2)
   id=64 ; i = a.8 / b.  ;   self~assertSame(i, 22)
   id=65 ; i = a.8 || 4  ;   self~assertSame(i, 884)
   id=66 ; i = a.8 || b  ;   self~assertSame(i, 884)
   id=67 ; i = a.8 || b.4  ;   self~assertSame(i, 8844)
   id=68 ; i = a.8 || b.  ;   self~assertSame(i, 884)
   id=69 ; i = a.8 = 4  ;   self~assertSame(i, 0)
   id=70 ; i = a.8 = b  ;   self~assertSame(i, 0)
   id=71 ; i = a.8 = b.4  ;   self~assertSame(i, 0)
   id=72 ; i = a.8 = b.  ;   self~assertSame(i, 0)
   id=73 ; i = a. + 4  ;   self~assertSame(i, 12)
   id=74 ; i = a. + b  ;   self~assertSame(i, 12)
   id=75 ; i = a. + b.4  ;   self~assertSame(i, 52)
   id=76 ; i = a. + b.  ;   self~assertSame(i, 12)
   id=77 ; i = a. - 4  ;   self~assertSame(i, 4)
   id=78 ; i = a. - b  ;   self~assertSame(i, 4)
   id=79 ; i = a. - b.4  ;   self~assertSame(i, -36)
   id=80 ; i = a. - b.  ;   self~assertSame(i, 4)
   id=81 ; i = a. * 4  ;   self~assertSame(i, 32)
   id=82 ; i = a. * b  ;   self~assertSame(i, 32)
   id=83 ; i = a. * b.4  ;   self~assertSame(i, 352)
   id=84 ; i = a. * b.  ;   self~assertSame(i, 32)
   id=85 ; i = a. / 4  ;   self~assertSame(i, 2)
   id=86 ; i = a. / b  ;   self~assertSame(i, 2)
   id=87 ; i = a. / b.4  ;   self~assertSame(i, 0.181818182)
   id=88 ; i = a. / b.  ;   self~assertSame(i, 2)
   id=89 ; i = a. || 4  ;   self~assertSame(i, 84)
   id=90 ; i = a. || b  ;   self~assertSame(i, 84)
   id=91 ; i = a. || b.4  ;   self~assertSame(i, 844)
   id=92 ; i = a. || b.  ;   self~assertSame(i, 84)
   id=93 ; i = a. = 4  ;   self~assertSame(i, 0)
   id=94 ; i = a. = b  ;   self~assertSame(i, 0)
   id=95 ; i = a. = b.4  ;   self~assertSame(i, 0)
   id=96 ; i = a. = b.  ;   self~assertSame(i, 0)

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
