/* extracted from yaml::test19_ArrayRoundtrip */
::routine main public
  expose parser thisLocation

  inFile = thisLocation"test_all_constructs.yaml"
  arrIn = parser~readFileLines(inFile)

  doc5  = parser~parseArray(arrIn)
  arrOut = .Yaml~toYamlArray(doc5)
  doc6  = parser~parseArray(arrOut)

  self~assertTrue(YAML.deepEqual(doc5, doc6), "array roundtrip equal")
  self~assertEquals("hello world", doc6["plain"], "array roundtrip plain")
  self~assertEquals(42, doc6["integer"], "array roundtrip integer")
  self~assertEquals(3, doc6["simple_list"]~items, "array roundtrip list")
  self~assertEquals("deep", doc6["nested_map"]["level1"]["level2"]["level3"], "array roundtrip nested")
  self~assertEquals("Bob", doc6["list_of_maps"][2]["name"], "array roundtrip lom name")

  arrOut2 = .Yaml~toYamlArray(doc6)
  self~assertEquals(arrOut~items, arrOut2~items, "array roundtrip stable count")
  doc7 = parser~parseArray(arrOut2)
  self~assertTrue(YAML.deepEqual(doc6, doc7), "array roundtrip stable")

/*------------------------------------------------------------------------*/
/* 20. Front-matter file round-trip                                       */
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
