/* extracted from yaml::test32_MergeKeyReconstruction */
::routine main public
  expose parser

  /* 32.1 Simple merge */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc, 2, am, msm)
  self~assertTrue(yamlOut~pos("<<: *defs") > 0, "merge key reconstructed")

  /* 32.2 Own keys not duplicated */
  self~assertTrue(yamlOut~countStr("adapter:") = 1, "adapter only in source")
  self~assertTrue(yamlOut~countStr("host:") = 1, "host only in source")
  self~assertTrue(yamlOut~pos("database:") > 0, "own key present")

  /* 32.3 Override */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  adapter: mysql"     || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc, 2, am, msm)
  self~assertTrue(yamlOut~countStr("adapter:") = 2, "overridden key emitted as own")
  self~assertTrue(yamlOut~countStr("host:") = 1, "non-overridden stays merged")
  self~assertTrue(yamlOut~pos("<<: *defs") > 0, "merge present with override")

  /* 32.4 Multiple merges */
  yaml = "base1: &b1"          || "0A"x || -
         "  x: 1"              || "0A"x || -
         "base2: &b2"          || "0A"x || -
         "  y: 2"              || "0A"x || -
         "combined:"            || "0A"x || -
         "  <<: [*b1, *b2]"    || "0A"x || -
         "  z: 3"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc, 2, am, msm)
  self~assertTrue(yamlOut~pos("<<: [*b1, *b2]") > 0, "multi merge reconstructed")
  self~assertTrue(yamlOut~countStr("x:") = 1, "multi merge x only in source")
  self~assertTrue(yamlOut~countStr("y:") = 1, "multi merge y only in source")
  self~assertTrue(yamlOut~pos("z:") > 0, "multi merge own key z present")

  /* 32.5 Round-trip simple merge */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc, 2, am, msm)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "merge simple roundtrip")

  /* 32.6 Round-trip with override */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  adapter: mysql"     || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc, 2, am, msm)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "merge override roundtrip")

  /* 32.7 Round-trip multiple merges */
  yaml = "base1: &b1"          || "0A"x || -
         "  x: 1"              || "0A"x || -
         "base2: &b2"          || "0A"x || -
         "  y: 2"              || "0A"x || -
         "combined:"            || "0A"x || -
         "  <<: [*b1, *b2]"    || "0A"x || -
         "  z: 3"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc, 2, am, msm)
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "multi merge roundtrip")

  /* 32.8 XML round-trip (XSD) */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  xml = .Yaml~yamlToXml(doc, "xsd", am)
  doc3 = parser~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc3), "merge xml xsd roundtrip")

  /* 32.9 XML round-trip (DTD) */
  xml = .Yaml~yamlToXml(doc, "dtd", am)
  doc4 = parser~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc4), "merge xml dtd roundtrip")

  /* 32.10 Without mergeSourceMap (backward compat) */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  yamlOut = .Yaml~toYaml(doc, 2, am)
  self~assertTrue(yamlOut~pos("<<:") = 0, "no merge without mergeSourceMap")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "roundtrip without mergeSourceMap")

  /* 32.11 P2: XML <merge> element emitted with mergeSourceMap (XSD) */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  xml = .Yaml~yamlToXml(doc, "xsd", am, , msm)
  self~assertTrue(xml~pos('<merge anchor="defs"/>') > 0, "merge element in XSD XML")

  /* 32.12 P2: XML <merge> element emitted with mergeSourceMap (DTD) */
  xml = .Yaml~yamlToXml(doc, "dtd", am, , msm)
  self~assertTrue(xml~pos('<merge anchor="defs"/>') > 0, "merge element in DTD XML")

  /* 32.13 P2: merged keys excluded from XML entries */
  xml = .Yaml~yamlToXml(doc, "xsd", am, , msm)
  mergePos = xml~pos('<merge anchor="defs"/>')
  xmlAfterMerge = xml~substr(mergePos)
  closeMappingPos = xmlAfterMerge~pos('</mapping>')
  xmlMergeSection = xmlAfterMerge~left(closeMappingPos)
  self~assertTrue(xmlMergeSection~countStr("<entry>") = 1, "only own entries in merge XML")

  /* 32.14 P2: XML without mergeSourceMap has no <merge> (backward compat) */
  xml = .Yaml~yamlToXml(doc, "xsd", am)
  self~assertTrue(xml~pos("<merge") = 0, "no merge element without mergeSourceMap")

  /* 32.15 P2: XML round-trip with <merge> preserves data (XSD) */
  xml = .Yaml~yamlToXml(doc, "xsd", am, , msm)
  parser2 = .Yaml~new
  doc3 = parser2~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc3), "merge XML xsd data roundtrip")

  /* 32.16 P2: XML round-trip with <merge> preserves data (DTD) */
  xml = .Yaml~yamlToXml(doc, "dtd", am, , msm)
  parser2 = .Yaml~new
  doc4 = parser2~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc4), "merge XML dtd data roundtrip")

  /* 32.17 P2: XML round-trip reconstructs mergeSourceMap */
  xml = .Yaml~yamlToXml(doc, "xsd", am, , msm)
  parser2 = .Yaml~new
  doc3 = parser2~parseXml(xml)
  am3  = parser2~anchorMap
  msm3 = parser2~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc3, 2, am3, msm3)
  self~assertTrue(yamlOut~pos("<<: *defs") > 0, "merge XML roundtrip reconstructs merge key")

  /* 32.18 P2: XML round-trip with merge override */
  yaml = "defaults: &defs"     || "0A"x || -
         "  adapter: postgres"  || "0A"x || -
         "  host: localhost"    || "0A"x || -
         "development:"         || "0A"x || -
         "  <<: *defs"          || "0A"x || -
         "  adapter: mysql"     || "0A"x || -
         "  database: myapp_dev"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  xml = .Yaml~yamlToXml(doc, "xsd", am, , msm)
  parser2 = .Yaml~new
  doc5 = parser2~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc5), "merge XML override data roundtrip")
  am5  = parser2~anchorMap
  msm5 = parser2~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc5, 2, am5, msm5)
  self~assertTrue(yamlOut~pos("<<: *defs") > 0, "merge XML override reconstructs merge")
  self~assertTrue(yamlOut~countStr("adapter:") = 2, "merge XML override own key emitted")

  /* 32.19 P2: XML round-trip with multiple merges */
  yaml = "base1: &b1"          || "0A"x || -
         "  x: 1"              || "0A"x || -
         "base2: &b2"          || "0A"x || -
         "  y: 2"              || "0A"x || -
         "combined:"            || "0A"x || -
         "  <<: [*b1, *b2]"    || "0A"x || -
         "  z: 3"
  doc = parser~parseString(yaml)
  am  = parser~anchorMap
  msm = parser~mergeSourceMap
  xml = .Yaml~yamlToXml(doc, "xsd", am, , msm)
  self~assertTrue(xml~countStr("<merge") = 2, "two merge elements in XML")
  parser2 = .Yaml~new
  doc6 = parser2~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc6), "multi merge XML data roundtrip")
  am6  = parser2~anchorMap
  msm6 = parser2~mergeSourceMap
  yamlOut = .Yaml~toYaml(doc6, 2, am6, msm6)
  self~assertTrue(yamlOut~pos("<<: [*b1, *b2]") > 0, "multi merge XML roundtrip reconstructs")

  /* 32.20 P2: XML round-trip with multiple merges (DTD) */
  xml = .Yaml~yamlToXml(doc, "dtd", am, , msm)
  parser2 = .Yaml~new
  doc7 = parser2~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc7), "multi merge XML dtd data roundtrip")

/*------------------------------------------------------------------------*/
/* 33. Single-quoted strings in emitter                                   */
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
