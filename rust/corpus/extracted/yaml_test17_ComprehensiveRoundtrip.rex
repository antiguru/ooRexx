/* extracted from yaml::test17_ComprehensiveRoundtrip */
::routine main public
  expose parser

  big = .table~new
  big["plain"] = "hello world"
  big["integer"] = 42
  big["negative"] = -17
  big["float"] = 3.14
  big["null_value"] = .nil
  big["empty_string"] = ""
  big["bool_true"] = "true"
  big["bool_false"] = "false"

  nested = .table~new
  nested["level2"] = .table~new
  nested["level2"]["deep"] = "value"
  big["nested_map"] = nested

  big["simple_list"] = .array~of("alpha", "beta", "gamma")

  lom = .array~new
  m1 = .table~new; m1["name"] = "Alice"; m1["age"] = 30
  m2 = .table~new; m2["name"] = "Bob";   m2["age"] = 25
  lom~append(m1); lom~append(m2)
  big["list_of_maps"] = lom

  big["empty_map"] = .table~new
  big["empty_list"] = .array~new

  multiline = "Line one" || "0A"x || "Line two" || "0A"x || "Line three"
  big["multiline"] = multiline

  big["special_chars"] = "tabs"||"09"x||"here and\backslash"
  big["comment-like"] = "value"
  big["key: colon"] = "value"

  yaml1 = .Yaml~toYaml(big)
  doc2  = parser~parseString(yaml1)
  self~assertTrue(doc2~isA(.table), "big roundtrip type")
  self~assertEquals("hello world", doc2["plain"], "big roundtrip plain")
  self~assertEquals(42, doc2["integer"], "big roundtrip integer")
  self~assertEquals(-17, doc2["negative"], "big roundtrip negative")
  self~assertEquals(3.14, doc2["float"], "big roundtrip float")
  self~assertEquals(.nil, doc2["null_value"], "big roundtrip null")
  self~assertEquals("", doc2["empty_string"], "big roundtrip empty str")
  self~assertEquals("true", doc2["bool_true"], "big roundtrip bool")
  self~assertEquals("value", doc2["nested_map"]["level2"]["deep"], "big roundtrip nested")
  self~assertEquals(3, doc2["simple_list"]~items, "big roundtrip list")
  self~assertEquals("beta", doc2["simple_list"][2], "big roundtrip list[2]")
  self~assertEquals("Alice", doc2["list_of_maps"][1]["name"], "big roundtrip lom")
  self~assertTrue(doc2["empty_map"]~isA(.table), "big roundtrip empty map")
  self~assertEquals(0, doc2["empty_map"]~items, "big roundtrip empty map size")
  self~assertTrue(doc2["empty_list"]~isA(.array), "big roundtrip empty list")
  self~assertEquals(0, doc2["empty_list"]~items, "big roundtrip empty list size")
  self~assertTrue(doc2["multiline"]~pos("Line two") > 0, "big roundtrip multiline")
  self~assertTrue(doc2["special_chars"]~pos("09"x) > 0, "big roundtrip special")
  self~assertEquals("value", doc2["comment-like"], "big roundtrip comment key")
  self~assertEquals("value", doc2["key: colon"], "big roundtrip colon key")

  yaml2 = .Yaml~toYaml(doc2)
  self~assertEquals(yaml1, yaml2, "big roundtrip stable")

/*------------------------------------------------------------------------*/
/* 18. File round-trip (toYamlFile / parseFile)                           */
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
