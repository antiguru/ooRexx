/* extracted from yaml::test18_FileRoundtrip */
::routine main public
  expose parser thisLocation

  inFile  = thisLocation"test_all_constructs.yaml"
  outFile = thisLocation"test18_roundtrip.yaml"

  doc1 = parser~parseFile(inFile)
  .Yaml~toYamlFile(doc1, outFile)
  doc3 = parser~parseFile(outFile)

  self~assertTrue(YAML.deepEqual(doc1, doc3), "file roundtrip equal")
  self~assertEquals("hello world", doc3["plain"], "file roundtrip plain")
  self~assertEquals(42, doc3["integer"], "file roundtrip integer")
  self~assertEquals("deep", doc3["nested_map"]["level1"]["level2"]["level3"], "file roundtrip nested")
  self~assertEquals("beta", doc3["simple_list"][2], "file roundtrip list")
  self~assertEquals("Alice", doc3["list_of_maps"][1]["name"], "file roundtrip lom")
  self~assertEquals(0, doc3["empty_map"]~items, "file roundtrip empty m")
  self~assertEquals(0, doc3["empty_list"]~items, "file roundtrip empty l")
  self~assertTrue(doc3["multiline"]~pos("multi-line") > 0, "file roundtrip multi")
  self~assertEquals("admin", doc3["mixed_nesting"]["users"][1]["name"], "file roundtrip users")
  self~assertEquals("write", doc3["mixed_nesting"]["users"][1]["roles"][2], "file roundtrip roles")

  .Yaml~toYamlFile(doc3, outFile)
  doc4 = parser~parseFile(outFile)
  self~assertTrue(YAML.deepEqual(doc3, doc4), "file roundtrip stable")

  -- cleanup
  call SysFileDelete outFile

/*------------------------------------------------------------------------*/
/* 19. Array round-trip (parseArray / toYamlArray)                        */
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
