/* extracted from Package_Options::test_options */
::routine main public
   -- get package with standard OPTIONS set
  pkg=.routine~new("package4options", "return .context~package")~call

   -- ~options
  expected="::OPTIONS DIGITS 9 FORM SCIENTIFIC FUZZ 0 NUMERIC NOINHERIT ERROR CONDITION FAILURE CONDITION LOSTDIGITS CONDITION NOSTRING CONDITION NOTREADY CONDITION NOVALUE CONDITION PROLOG TRACE NORMAL"
  self~assertEquals(expected, pkg~options)
  self~assertEquals(9,pkg~options("DIGITS"))
  self~assertEquals(9,pkg~options("D",6))
  self~assertEquals(6,pkg~options("DIGITS"))

  self~assertEquals("SCIENTIFIC" ,pkg~options("FORM"))
  self~assertEquals("SCIENTIFIC" ,pkg~options("FO","E"))
  self~assertEquals("ENGINEERING",pkg~options("fo"))

  self~assertEquals(0,pkg~options("fuzz"))
  self~assertEquals(0,pkg~options("Fu","4"))
  self~assertEquals(4,pkg~options("fu"))

  self~assertEquals("CONDITION",pkg~options("error"))
  self~assertEquals("CONDITION",pkg~options("e","s"))
  self~assertEquals("SYNTAX"   ,pkg~options("E"))

  self~assertEquals("CONDITION",pkg~options("failure"))
  self~assertEquals("CONDITION",pkg~options("fa","s"))
  self~assertEquals("SYNTAX"   ,pkg~options("FA"))

  self~assertEquals("CONDITION",pkg~options("lostdigits"))
  self~assertEquals("CONDITION",pkg~options("l","s"))
  self~assertEquals("SYNTAX"   ,pkg~options("l"))

  self~assertEquals("CONDITION",pkg~options("nostring"))
  self~assertEquals("CONDITION",pkg~options("nos","s"))
  self~assertEquals("SYNTAX"   ,pkg~options("NOS"))

  self~assertEquals("CONDITION",pkg~options("novalue"))
  self~assertEquals("CONDITION",pkg~options("nov","s"))
  self~assertEquals("SYNTAX"   ,pkg~options("NOV"))

  self~assertEquals("CONDITION",pkg~options("notready"))
  self~assertEquals("CONDITION",pkg~options("not","s"))
  self~assertEquals("SYNTAX"   ,pkg~options("NOt"))

  self~assertEquals("PROLOG"  ,pkg~options("proplog"))
  self~assertEquals("PROLOG"  ,pkg~options("prolog","n"))
  self~assertEquals("NOPROLOG",pkg~options("prolog","p"))
  self~assertEquals("PROLOG"  ,pkg~options("prolog","n"))
  self~assertEquals("NOPROLOG",pkg~options("prolog"))

  self~assertEquals("NORMAL"       ,pkg~options("trace"))
  self~assertEquals("NORMAL"       ,pkg~options("trace","all"))
  self~assertEquals("ALL"          ,pkg~options("trace","results"))
  self~assertEquals("RESULTS"      ,pkg~options("trace","intermediates"))
  self~assertEquals("INTERMEDIATES",pkg~options("trace","labels"))
  self~assertEquals("LABELS"       ,pkg~options("trace","error"))
  self~assertEquals("ERROR"        ,pkg~options("trace","failure"))
  self~assertEquals("FAILURE"      ,pkg~options("trace","commands"))
  self~assertEquals("COMMANDS"     ,pkg~options("trace","off"))
  self~assertEquals("OFF"          ,pkg~options("trace","normal"))

  self~assertEquals("NORMAL"       ,pkg~options("trace"))
  self~assertEquals("NORMAL"       ,pkg~options("trace","a"))
  self~assertEquals("ALL"          ,pkg~options("trace","r"))
  self~assertEquals("RESULTS"      ,pkg~options("trace","i"))
  self~assertEquals("INTERMEDIATES",pkg~options("trace","l"))
  self~assertEquals("LABELS"       ,pkg~options("trace","e"))
  self~assertEquals("ERROR"        ,pkg~options("trace","f"))
  self~assertEquals("FAILURE"      ,pkg~options("trace","c"))
  self~assertEquals("COMMANDS"     ,pkg~options("trace","o"))
  self~assertEquals("OFF"          ,pkg~options("trace","n"))

  options_02="::OPTIONS DIGITS 6 FORM ENGINEERING FUZZ 4 NUMERIC NOINHERIT ERROR SYNTAX FAILURE SYNTAX LOSTDIGITS SYNTAX NOSTRING SYNTAX NOTREADY SYNTAX NOVALUE SYNTAX NOPROLOG TRACE NORMAL"
  self~assertEquals(options_02, pkg~options)

  curr4all_1="ERROR SYNTAX FAILURE SYNTAX LOSTDIGITS SYNTAX NOSTRING SYNTAX NOTREADY SYNTAX NOVALUE SYNTAX"
  self~assertEquals(curr4all_1,pkg~options("all","condition"))
  curr4all_2="ERROR CONDITION FAILURE CONDITION LOSTDIGITS CONDITION NOSTRING CONDITION NOTREADY CONDITION NOVALUE CONDITION"
  self~assertEquals(curr4all_2,pkg~options("all","syntax"))

  self~assertEquals(curr4all_1,pkg~options("a","c"))
  self~assertEquals(curr4all_2,pkg~options("a","s"))

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
