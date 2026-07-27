/* extracted from yaml::test26_ComprehensiveFileRoundtrip */
::routine main public
  expose parser thisLocation

  inFile = thisLocation"test_all_constructs.yaml"
  doc1 = parser~parseFile(inFile)
  am1  = parser~anchorMap

  self~assertTrue(am1~items > 0, "file anchor captured")

  yamlOut = .Yaml~toYaml(doc1)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc1, doc2), "file yaml rt equal")

  self~assertTrue(doc2["bool_true"]~isA(.YamlBoolean), "file bool true isA")
  self~assertTrue(doc2["bool_false"]~isA(.YamlBoolean), "file bool false isA")
  self~assertTrue(doc2["bool_yes"]~isA(.YamlBoolean), "file bool yes isA")
  self~assertTrue(doc2["bool_no"]~isA(.YamlBoolean), "file bool no isA")
  self~assertTrue(doc2["bool_on"]~isA(.YamlBoolean), "file bool on isA")
  self~assertTrue(doc2["bool_off"]~isA(.YamlBoolean), "file bool off isA")

  self~assertEquals(255, doc2["hex_value"], "file hex value")
  self~assertEquals(63, doc2["octal_value"], "file octal value")
  self~assertEquals(".inf", doc2["infinity"], "file infinity")
  self~assertEquals("-.inf", doc2["neg_infinity"], "file neg infinity")
  self~assertEquals(".nan", doc2["not_a_number"], "file nan")
  self~assertEquals(6.022E23, doc2["scientific"], "file scientific")

  self~assertEquals(3, doc2["flow_sequence"]~items, "file flow seq")
  self~assertEquals("two", doc2["flow_sequence"][2], "file flow seq 2")
  self~assertEquals(10, doc2["flow_mapping"]["x"], "file flow map x")
  self~assertEquals(2, doc2["nested_flow"][1][2], "file nested flow")
  self~assertEquals(1, doc2["mixed_flow"][1]["a"], "file mixed flow")

  self~assertTrue(doc2["folded"]~pos("folded block scalar") > 0, "file folded")
  self~assertTrue(doc2["strip_chomp"]~right(1) \== "0A"x, "file strip chomp")
  self~assertTrue(doc2["multiline"]~pos("multi-line") > 0, "file multiline")

  self~assertEquals("It's a test", doc2["single_quoted"], "file single quoted")
  self~assertTrue(doc2["double_quoted"]~pos("09"x) > 0, "file double quoted")

  self~assertEquals("deep", doc2["deeply_nested_list"][1][1][1], "file deeply nested")
  self~assertEquals("zero", doc2["numeric_keys"]["0"], "file numeric key 0")
  self~assertEquals("pi", doc2["numeric_keys"]["3.14"], "file numeric key pi")
  self~assertEquals("  two spaces", doc2["leading_space_value"], "file leading spaces")
  self~assertEquals("value  ", doc2["trailing_space_value"], "file trailing spaces")
  self~assertEquals("http://example.com", doc2["colon_in_value"], "file colon in value")
  self~assertEquals("color #FF0000", doc2["hash_in_value"], "file hash in value")
  self~assertEquals("only_value", doc2["single_item_map"]["only_key"], "file single item map")
  self~assertEquals("only", doc2["single_item_list"][1], "file single item list")

  self~assertTrue(doc2["null_value"] == .nil, "file null value")
  self~assertTrue(doc2["null_tilde"] == .nil, "file null tilde")

  self~assertEquals("true", doc2["true"], "file str true key")
  self~assertEquals("false", doc2["false"], "file str false key")
  self~assertEquals("null", doc2["null"], "file str null key")
  self~assertEquals("NULL", doc2["NULL"], "file str NULL key")

  self~assertEquals("postgres", doc2["development"]["adapter"], "file merge adapter")
  self~assertEquals("localhost", doc2["development"]["host"], "file merge host")
  self~assertEquals("myapp_dev", doc2["development"]["database"], "file merge database")

  /* Verify keep chomp (|+) */
  self~assertTrue(doc2["keep_chomp"]~right(1) == "0A"x, "file keep chomp")

  /* Verify folded strip (>-) and folded keep (>+) */
  self~assertTrue(doc2["folded_strip"]~right(1) \== "0A"x, "file folded strip")
  self~assertTrue(doc2["folded_strip"]~pos("folded without trailing newline") > 0, "file folded strip content")
  self~assertTrue(doc2["folded_keep"]~right(1) == "0A"x, "file folded keep")
  self~assertTrue(doc2["folded_keep"]~pos("folded with trailing newlines kept") > 0, "file folded keep content")

  /* Verify Unicode escapes (unescaped to UTF-8 by default) */
  self~assertEquals("C3A9", doc2["unicode_escape_2byte"]~c2x, "file unicode 2byte")
  self~assertEquals(3, doc2["unicode_escape_3byte"]~length, "file unicode 3byte len")
  self~assertEquals(4, doc2["unicode_escape_8digit"]~length, "file unicode 8digit len")

  /* Verify tags — values resolved, tags stripped */
  self~assertEquals(1, doc2["tagged_map"]["x"], "file tagged map x")
  self~assertEquals(2, doc2["tagged_map"]["y"], "file tagged map y")
  self~assertEquals("42", doc2["tagged_flow"][1], "file tagged flow str")
  self~assertEquals(7, doc2["tagged_flow"][2], "file tagged flow int")
  self~assertEquals("value123", doc2["custom_tag"], "file custom tag")
  self~assertEquals("hello", doc2["verbatim_tag"], "file verbatim tag")

  /* Verify complex mapping keys */
  self~assertEquals("scalar_complex_value", doc2["scalar_key"], "file scalar complex key")
  complexFound = 0
  sup = doc2~supplier
  Do While sup~available
    If sup~index~isA(.array) Then Do
      If sup~index~items = 2, sup~index[1] = "a", sup~index[2] = "b" Then
        complexFound += 1
    End
    If sup~index~isA(.table) Then Do
      If sup~index["x"] = 1 Then
        complexFound += 1
    End
    sup~next
  End
  self~assertEquals(2, complexFound, "file complex keys found")

  xml1 = .Yaml~yamlToXml(doc1, "xsd")
  doc3 = parser~parseXml(xml1)
  self~assertTrue(YAML.deepEqual(doc1, doc3), "file xml xsd rt equal")

  xml2 = .Yaml~yamlToXml(doc1, "dtd")
  doc4 = parser~parseXml(xml2)
  self~assertTrue(YAML.deepEqual(doc1, doc4), "file xml dtd rt equal")

  doc5 = parser~parseString(.Yaml~toYaml(doc3))
  self~assertTrue(YAML.deepEqual(doc1, doc5), "file chain rt equal")

/*------------------------------------------------------------------------*/
/* 27. Multi-document with anchorMap                                      */
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
