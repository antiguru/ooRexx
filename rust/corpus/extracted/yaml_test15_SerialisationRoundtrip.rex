/* extracted from yaml::test15_SerialisationRoundtrip */
::routine main public
  expose parser

  yaml = "name: Test"             || "0A"x || -
         "items:"                  || "0A"x || -
         "  - alpha"               || "0A"x || -
         "  - beta"                || "0A"x || -
         "nested:"                 || "0A"x || -
         "  key: value"

  doc  = parser~parseString(yaml)
  out  = .Yaml~toYaml(doc)
  doc2 = parser~parseString(out)
  self~assertEquals("Test", doc2["name"], "roundtrip name")
  self~assertEquals(2, doc2["items"]~items, "roundtrip items")
  self~assertEquals("value", doc2["nested"]["key"], "roundtrip nested")

/*------------------------------------------------------------------------*/
/* 16. Pandoc-style front matter                                          */
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
