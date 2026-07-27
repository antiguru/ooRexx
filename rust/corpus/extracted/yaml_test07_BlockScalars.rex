/* extracted from yaml::test07_BlockScalars */
::routine main public
  expose parser

  yaml = "literal: |"    || "0A"x || -
         "  line one"    || "0A"x || -
         "  line two"    || "0A"x || -
         "folded: >"     || "0A"x || -
         "  this is"     || "0A"x || -
         "  one line"    || "0A"x || -
         "stripped: |-"  || "0A"x || -
         "  no newline"

  doc = parser~parseString(yaml)
  self~assertTrue(doc["literal"]~countStr("0A"x) >= 1, "literal has newline")
  self~assertTrue(doc["folded"]~pos("this is one line") > 0, "folded joins")
  self~assertTrue(doc["stripped"]~right(1) \== "0A"x, "strip chomp")

/*------------------------------------------------------------------------*/
/* 8. Anchors & aliases                                                   */
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
