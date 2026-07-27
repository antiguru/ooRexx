/* extracted from C2D::test_91 */
::routine main public
   NUMERIC DIGITS 5000

   va='8689773497565926399095561091266377369266953588357575438909217576'
   vb='7597211156363713950719717983955180429121596539342972672656978023'
   vc='3123802662608646085255545352080474035491971374122860171592561908'
   vd='2425797219247942292252098739013236452643855759225807845614268495'
   ve='8179315070471083582407393329993570508607216761598036959213466668'
   vf='5944363107473252055426191388700571874412718603089619245002726337'
   vg='3446788205765286361126904357454391574589612812541230137907595116'
   vh='6241806835700943792575891109045896305473795069841970477601655441'
   vi='8004744979267285449034765498848950050562614072904198374855241363'
   vj='44577774808241867732402625'
   l250r=va||vb||vc||vd||ve||vf||vg||vh||vi||vj
   wa='2926607654620648787260163061825463880841201985716281883570461878'
   wb='1833257643334929568895241808067806880274112824131052972656495189'
   wc='1984700378598766816277774341374252913507658752942932182142572663'
   wd='3977600099746716316043452943190986370061402198702992279611023068'
   we='5397178650676789807546531432381254316370306163232499494035623489'
   wf='4229966849667157427993276892567550113144179840418783683550140994'
   wg='4425084110749966909187817529712618924084584625985388388414475039'
   wh='2775738053474411121851984052269343315315008702278383839606775148'
   wi='7929059189908671265218962473705604939308652407972916551117050200'
   wj='12422566645262805194850625'
   a250r=wa||wb||wc||wd||we||wf||wg||wh||wi||wj
   self~assertSame(C2D(copies('A',250)), a250r)
   self~assertSame(C2D(copies('C1'x,250)), l250r)
   return

-- rexxref documentation tests

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
