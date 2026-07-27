/* extracted from Package_Options::test_option_x */
::routine main public
  test_options = ("::OPTIONS DIGITS 8"               ,"DIGITS"      ), -
                 ("::OPTIONS FUZZ 2"                 ,"FUZZ"        ), -
                 ("::OPTIONS FORM ENGINEERING"       ,"FORM"        ), -
                 ("::OPTIONS NUMERIC INHERIT"        ,"NUMERIC"     ), -
                 ("::OPTIONS ALL SYNTAX"             ,"ERROR FAILURE LOSTDIGITS NOSTRING NOTREADY NOVALUE"), -
                 ("::OPTIONS ERROR SYNTAX"           ,"ERROR"       ), -
                 ("::OPTIONS FAILURE SYNTAX"         ,"FAILURE"     ), -
                 ("::OPTIONS LOSTDIGITS SYNTAX"      ,"LOSTDIGITS"  ), -
                 ("::OPTIONS NOSTRING SYNTAX"        ,"NOSTRING"    ), -
                 ("::OPTIONS NOTREADY SYNTAX"        ,"NOTREADY"    ), -
                 ("::OPTIONS NOVALUE SYNTAX"         ,"NOVALUE"     ), -
                 ("::OPTIONS NOPROLOG"               ,"NOPROLOG"    ), -
                 ("::OPTIONS TRACE OFF"              ,"TRACE"       )

  newLine = .endOfLine
  test_options_items = test_options~items
  base_code=("use arg nr, testcase, arr;"                                  , -
             "testcase~assertSame(arr[2], .context~package~options('X')); ", -
             "return .nil;")

  fn_base=.context~name
  do counter c arr over test_options
     code=base_code~copy~~append(arr[1])
     name=fn_base"_"c~right(2,0)
     routine=.routine~new(name, code)
     routine~callWith((c,self,arr))
  end





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
