/* extracted from TIME::test_7 */
::routine main public
   Call time 'r'                       /* reset the timer for the next  */
   xxyy=1                              /* do very little work           */
   tx=TIME( )||'**'||TIME('life')||'*'||TIME('honour')||'*'||,
    TIME('micro')||'*'||TIME('short')||'*'||,
    TIME('europe')||'*'||TIME('rage')||'*'||,
    TIME('carbon')||'**'||,
    TIME('LIFE')||'*'||TIME('HONOUR')||'*'||,
    TIME('MICRO')||'*'||TIME('SHORT')||'*'||,
    TIME('EUROPE')||'*'||TIME('RAGE')||'*'||,
    TIME('CARBON')
   Parse Var tx tt '**' tx '**' ty
   Parse Var tt tth ':' ttm ':' tts
   Parse Var tx txl '*' txh '*' txm '*' txs '*' txe '*' txr '*' txc
   Parse Var ty tyl '*' tyh '*' tym '*' tys '*' tye '*' tyr '*' tyc
   v.1='life'; v.2='honour'; v.3='micro'; v.4='short'; v.5='europe';
   v.6='rage'; v.7='carbon'; v.8='LIFE'; v.9='HONOUR'; v.a='MICRO';
   v.b='SHORT'; v.c='EUROPE'; v.d='RAGE'; v.e='CARBON'
   vx=TIME( )||'**'||TIME(v.1)||'*'||TIME(v.2)||'*'||TIME(v.3)||'*'||,
    TIME(v.4)||'*'||TIME(v.5)||'*'||TIME(v.6)||'*'||,
    TIME(v.7)||'**'||,
    TIME(v.8)||'*'||TIME(v.9)||'*'||TIME(v.a)||'*'||,
    TIME(v.b)||'*'||TIME(v.c)||'*'||TIME(v.d)||'*'||,
    TIME(v.e)
   Parse Var vx vt '**' vx '**' vy
   Parse Var vt vth ':' vtm ':' vts
   Parse Var vx vxl '*' vxh '*' vxm '*' vxs '*' vxe '*' vxr '*' vxc
   Parse Var vy vyl '*' vyh '*' vym '*' vys '*' vye '*' vyr '*' vyc

   self~assertSame(tx, ty)
   self~assertSame(txm, ttm+60*tth)
   self~assertSame(txs, tts+60*(ttm+60*tth))
   If tth<10 Then self~assertSame(txh, right(tth, 1))
   else self~assertSame(txh, tth)

   self~assertSame(vx, vy)
   self~assertSame(vxm, vtm+60*vth)
   self~assertSame(vxs, vts+60*(vtm+60*vth))
   If vth<10 Then self~assertSame(vxh, right(vth, 1))
   else self~assertSame(vxh, vth)

   self~assertSame(txe, txr)
   self~assertTrue(txe<='1')

   self~assertSame(vxe, vxr)

   self~assertSame(txl, tyl)
   self~assertSame(vxl, vyl)

   Parse Var txl txlh ':' txlm ':' txls '.' txlms
   Parse Var vxl vxlh ':' vxlm ':' vxls '.' vxlms
   tmsc=SUBSTR(txl,9,7)
   vmsc=SUBSTR(vxl,9,7)
   Numeric Digits 20
   txlc=txls+60*(txlm+60*txlh)+tmsc
   vxlc=vxls+60*(vxlm+60*vxlh)+vmsc
   xlc=vxlc-txlc

   self~assertSame(vxr, xlc)


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
