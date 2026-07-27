/* extracted from yaml::test06_TypeResolution */
::routine main public
  expose parser

  yaml = "null1: null"   || "0A"x || -
         "null2: ~"      || "0A"x || -
         "btrue: true"   || "0A"x || -
         "bfalse: no"    || "0A"x || -
         "int: 42"       || "0A"x || -
         "neg: -17"      || "0A"x || -
         "hex: 0xFF"     || "0A"x || -
         "oct: 0o77"     || "0A"x || -
         "float: 3.14"   || "0A"x || -
         "sci: 1.5e+3"   || "0A"x || -
         "inf: .inf"     || "0A"x || -
         "nan: .nan"     || "0A"x || -
         'strnull: "null"'

  doc = parser~parseString(yaml)
  self~assertEquals(1, doc["null1"] == .nil, "null")
  self~assertEquals(1, doc["null2"] == .nil, "tilde")
  self~assertEquals(1, doc["btrue"], "true")
  self~assertEquals(0, doc["bfalse"], "false/no")
  self~assertEquals(42, doc["int"], "int")
  self~assertEquals(-17, doc["neg"], "negative")
  self~assertEquals(255, doc["hex"], "hex")
  self~assertEquals(63, doc["oct"], "octal")
  self~assertEquals(3.14, doc["float"], "float")
  self~assertEquals(".inf", doc["inf"], "inf")
  self~assertEquals(".nan", doc["nan"], "nan")
  self~assertEquals("null", doc["strnull"], "str null")

/*------------------------------------------------------------------------*/
/* 7. Block scalars                                                       */
/*------------------------------------------------------------------------*/
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
