/* extracted from ASSIGNMENT::test_2 */
::routine main public
   a=''
   id=0003; b = ''               ; self~assertSame(b, '')
   id=0004; c = ''x              ; self~assertSame(c, '')
   id=0005; xi=1 ; d = ''xi      ; self~assertSame(d, '1')
   id=0006; e = a||b||c||d       ; self~assertSame(e, 1)
   id=0007; f = substr('abc',4)  ; self~assertSame(f, '')
   id=0010; i = 'IBM'            ; self~assertSame(i, 'IBM')
   id=0011; j = 1bcde            ; self~assertSame(j, '1BCDE')
   id=0015; l = abcde            ; self~assertSame(l, 'ABCDE')
   Drop a
   achar='A'
   id=0018; stem.a = 'val'       ; self~assertSame(stem.ACHAR, 'val')
   id=0019; stem.  = 'all'       ; self~assertSame(stem.ACHAR, 'all')
                                   self~assertSame(stem., 'all')
                                   self~assertSame(stem.noval, 'all')
   tail='R.S.T.U'
   id=0020; stem.tail=4711       ; self~assertSame(stem.r.s.t.u, 4711)
   id=0021; x=stem.              ; self~assertSame(x, 'all')
   nil=''
   id=0022; x=stem.nil           ; self~assertSame(x, 'all')
   id=0025; x=reverse('abc')     ; self~assertSame(x, 'cba')
   term.=1; term.2=2; term.3=3; term.4=4
   id=0026; x=term.*2+term.2      ; self~assertSame(x, 4)
   id=0027; x=term.2*(2+term.)    ; self~assertSame(x, 6)
   id=0028; y=((term.2*(2+term.))); self~assertSame(y, 6)
   id=0029;
   xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx=1
   z=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx+,
     xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx+,
     xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx+,
     xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx+,
     xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                                    self~assertSame(z, 5)
   id=0030; zz=z=z;               ; self~assertSame(zz, 1)
   id=0032; zzz=(1=(0=(0=1)))     ; self~assertSame(zzz, 1)
   id=0033; zzz=(((1=0)=0)=1)     ; self~assertSame(zzz, 1)

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
