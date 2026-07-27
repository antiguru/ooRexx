/* extracted from yaml::test33_SingleQuotedStrings */
::routine main public
  expose parser

  /* Reserved words should now be single-quoted */
  doc = .table~new
  doc["a"] = "true"; doc["b"] = "false"; doc["c"] = "null"
  doc["d"] = "yes"; doc["e"] = "no"
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("'true'") > 0, "single-quote true")
  self~assertTrue(yamlOut~pos("'false'") > 0, "single-quote false")
  self~assertTrue(yamlOut~pos("'null'") > 0, "single-quote null")
  self~assertTrue(yamlOut~pos("'yes'") > 0, "single-quote yes")

  /* Strings needing quoting but no escapes use single quotes */
  doc2 = .table~new
  doc2["k"] = "key: colon"; doc2["h"] = "color #FF0000"
  yamlOut2 = .Yaml~toYaml(doc2)
  self~assertTrue(yamlOut2~pos("'key: colon'") > 0, -
    "single-quote colon in value")
  self~assertTrue(yamlOut2~pos("'color #FF0000'") > 0, -
    "single-quote hash in value")

  /* Strings with backslash still use double quotes */
  doc3 = .table~new
  doc3["p"] = "C:\path\to"
  yamlOut3 = .Yaml~toYaml(doc3)
  self~assertTrue(yamlOut3~pos('"C:\\path\\to"') > 0, -
    "double-quote backslash")

  /* Strings with control chars still use double quotes */
  doc4 = .table~new
  doc4["t"] = "tab" || "09"x || "here"
  yamlOut4 = .Yaml~toYaml(doc4)
  self~assertTrue(yamlOut4~pos('"tab\there"') > 0, -
    "double-quote control char")

  /* Round-trip */
  doc5 = .table~new
  doc5["w1"] = "true"; doc5["w2"] = "null"
  doc5["w3"] = "key: colon"; doc5["w4"] = "has 'apostrophe' here"
  yamlOut5 = .Yaml~toYaml(doc5)
  doc5b = parser~parseString(yamlOut5)
  self~assertEquals("true", doc5b["w1"], "single-quote roundtrip true")
  self~assertEquals("null", doc5b["w2"], "single-quote roundtrip null")
  self~assertEquals("key: colon", doc5b["w3"], -
    "single-quote roundtrip colon")
  self~assertEquals("has 'apostrophe' here", doc5b["w4"], -
    "single-quote roundtrip apostrophe")

  /* Keys: reserved words as keys use single quotes */
  doc6 = .table~new
  doc6["true"] = 1; doc6["null"] = 2
  yamlOut6 = .Yaml~toYaml(doc6)
  self~assertTrue(yamlOut6~pos("'true':") > 0, "single-quote key true")
  self~assertTrue(yamlOut6~pos("'null':") > 0, "single-quote key null")

/*------------------------------------------------------------------------*/
/* 34. Flow mappings in emitter                                           */
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
