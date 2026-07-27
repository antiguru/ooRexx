/* extracted from yaml::test27_MultiDocumentAnchorMap */
::routine main public
  expose parser

  yamlMulti = "---"                     || "0A"x || -
              "name: first"             || "0A"x || -
              "---"                     || "0A"x || -
              "name: second"

  docs = parser~parseAll(yamlMulti)
  self~assertEquals(2, docs~items, "multi doc count")
  self~assertEquals("first", docs[1]["name"], "multi doc 1")
  self~assertEquals("second", docs[2]["name"], "multi doc 2")

  multiOut = .Yaml~toYamlAll(docs)
  self~assertTrue(multiOut~countStr("---") >= 2, "multi has ---")
  docs2 = parser~parseAll(multiOut)
  self~assertEquals(2, docs2~items, "multi rt count")
  self~assertEquals("first", docs2[1]["name"], "multi rt doc 1")
  self~assertEquals("second", docs2[2]["name"], "multi rt doc 2")
  self~assertTrue(YAML.deepEqual(docs[1], docs2[1]), "multi rt equal 1")
  self~assertTrue(YAML.deepEqual(docs[2], docs2[2]), "multi rt equal 2")

/*------------------------------------------------------------------------*/
/* 28. Escape character round-trips                                       */
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
