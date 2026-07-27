/* extracted from yaml::test25_AnchorAliasRoundtrip */
::routine main public
  expose parser

  yamlAnch = "defaults: &defs"     || "0A"x || -
             "  adapter: postgres"  || "0A"x || -
             "  host: localhost"    || "0A"x || -
             "other: &other"        || "0A"x || -
             "  port: 5432"         || "0A"x || -
             "development:"         || "0A"x || -
             "  <<: *defs"          || "0A"x || -
             "  database: myapp_dev"

  docA = parser~parseString(yamlAnch)
  am   = parser~anchorMap

  self~assertTrue(am~items > 0, "anchorMap has entries")
  self~assertEquals("postgres", docA["development"]["adapter"], "merge adapter")
  self~assertEquals("myapp_dev", docA["development"]["database"], "merge database")

  yamlShared = "base: &shared"        || "0A"x || -
               "  x: 10"              || "0A"x || -
               "  y: 20"              || "0A"x || -
               "ref1: *shared"        || "0A"x || -
               "ref2: *shared"

  docS = parser~parseString(yamlShared)
  amS  = parser~anchorMap

  self~assertTrue((docS["ref1"] == docS["base"]) | (docS["ref1"] == docS["ref2"]), "ref1 identity")
  self~assertEquals(10, docS["ref1"]["x"], "ref1 x")
  self~assertEquals(20, docS["ref2"]["y"], "ref2 y")

  yamlOut = .Yaml~toYaml(docS, 2, amS)
  self~assertTrue(yamlOut~pos("&shared") > 0, "yaml has anchor")
  self~assertTrue(yamlOut~pos("*shared") > 0, "yaml has alias")

  docS2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(docS, docS2), "yaml rt equal")

  yamlExpanded = .Yaml~toYaml(docS)
  self~assertTrue(yamlExpanded~pos("*shared") = 0, "expanded no alias")

  xmlAnch = .Yaml~yamlToXml(docS, "xsd", amS)
  self~assertTrue(xmlAnch~pos('anchor="shared"') > 0, "xml has anchor attr")
  self~assertTrue(xmlAnch~pos('<alias') > 0, "xml has alias elem")

  docS3 = parser~parseXml(xmlAnch)
  self~assertTrue(YAML.deepEqual(docS, docS3), "xml rt equal")
  self~assertTrue((docS3["ref1"] == docS3["ref2"]) | -
                  (docS3["ref1"] == docS3["base"]) | -
                  (docS3["ref2"] == docS3["base"]), "xml identity")

  xmlDtd = .Yaml~yamlToXml(docS, "dtd", amS)
  self~assertTrue(xmlDtd~pos('anchor="shared"') > 0, "dtd has anchor")
  docS4 = parser~parseXml(xmlDtd)
  self~assertTrue(YAML.deepEqual(docS, docS4), "dtd rt equal")

  yamlSeq = "items:"             || "0A"x || -
            "  - &item1"         || "0A"x || -
            "    name: first"    || "0A"x || -
            "    val: 100"       || "0A"x || -
            "  - *item1"

  docSeq = parser~parseString(yamlSeq)
  amSeq  = parser~anchorMap
  self~assertEquals("first", docSeq["items"][2]["name"], "seq alias name")
  self~assertTrue(docSeq["items"][1] == docSeq["items"][2], "seq identity")

  xmlSeq = .Yaml~yamlToXml(docSeq, "xsd", amSeq)
  docSeq2 = parser~parseXml(xmlSeq)
  self~assertTrue(YAML.deepEqual(docSeq, docSeq2), "seq xml rt equal")
  self~assertTrue(docSeq2["items"][1] == docSeq2["items"][2], "seq xml identity")

/*------------------------------------------------------------------------*/
/* 26. Comprehensive file round-trip with all constructs                  */
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
