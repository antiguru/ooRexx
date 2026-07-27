/* extracted from yaml::test35_Directives */
::routine main public
  expose parser

  /* 35.1 %YAML directive is parsed */
  yaml = "%YAML 1.2" || "0A"x || "---" || "0A"x || "key: value"
  doc = parser~parseString(yaml)
  self~assertEquals("value", doc["key"], "YAML directive parsed")

  /* 35.2 %TAG directive is parsed */
  yaml = "%TAG ! tag:example.com,2000:" || "0A"x || -
         "---" || "0A"x || "name: test"
  doc = parser~parseString(yaml)
  self~assertEquals("test", doc["name"], "TAG directive parsed")

  /* 35.3 Multiple directives */
  yaml = "%YAML 1.2"                      || "0A"x || -
         "%TAG !! tag:yaml.org,2002:"     || "0A"x || -
         "%TAG !e! tag:example.com,2000:" || "0A"x || -
         "---"                             || "0A"x || -
         "color: red"
  doc = parser~parseString(yaml)
  self~assertEquals("red", doc["color"], "multiple directives parsed")

  /* 35.4 Directives with multi-document */
  yaml = "%YAML 1.2"   || "0A"x || -
         "---"          || "0A"x || -
         "first: 1"     || "0A"x || -
         "..."          || "0A"x || -
         "%YAML 1.2"    || "0A"x || -
         "---"          || "0A"x || -
         "second: 2"
  docs = parser~parseAll(yaml)
  self~assertEquals(2, docs~items, "directives multi-doc count")
  self~assertEquals(1, docs[1]["first"], "directives multi-doc first")
  self~assertEquals(2, docs[2]["second"], "directives multi-doc second")

  /* 35.5 Without directivesMap, directives do not appear in output */
  yaml = "%YAML 1.2" || "0A"x || "---" || "0A"x || "a: 1"
  doc = parser~parseString(yaml)
  yamlOut = .Yaml~toYaml(doc)
  self~assertTrue(yamlOut~pos("%YAML") = 0, "no directives without map")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "directives roundtrip no map")

  /* 35.6 directivesMap captures %YAML version */
  p6 = .Yaml~new
  yaml = "%YAML 1.2" || "0A"x || "---" || "0A"x || "x: 1"
  doc = p6~parseString(yaml)
  dir = p6~directivesMap~at(doc)
  self~assertTrue(dir \== .nil, "directivesMap has entry")
  self~assertEquals("1.2", dir["yamlVersion"], "yamlVersion captured")

  /* 35.7 directivesMap captures %TAG handles */
  p7 = .Yaml~new
  yaml = "%TAG !! tag:yaml.org,2002:"    || "0A"x || -
         "%TAG !e! tag:example.com,2000:" || "0A"x || -
         "---"                            || "0A"x || "a: 1"
  doc = p7~parseString(yaml)
  dir = p7~directivesMap~at(doc)
  self~assertTrue(dir \== .nil, "TAG directivesMap has entry")
  th = dir["tagHandles"]
  self~assertTrue(th \== .nil, "tagHandles captured")
  self~assertEquals("tag:yaml.org,2002:", th["!!"], "!! handle captured")
  self~assertEquals("tag:example.com,2000:", th["!e!"], "!e! handle captured")

  /* 35.8 Round-trip with directivesMap preserves directives */
  p8 = .Yaml~new
  yaml = "%YAML 1.2"                  || "0A"x || -
         "%TAG !! tag:yaml.org,2002:" || "0A"x || -
         "---"                         || "0A"x || -
         "color: red"
  doc = p8~parseString(yaml)
  dirMap = p8~directivesMap
  yamlOut = .Yaml~toYaml(doc, 2, .nil, .nil, dirMap)
  self~assertTrue(yamlOut~pos("%YAML 1.2") > 0, "YAML directive in output")
  self~assertTrue(yamlOut~pos("%TAG !!") > 0, "TAG directive in output")
  self~assertTrue(yamlOut~pos("---") > 0, "doc-start marker in output")
  doc2 = parser~parseString(yamlOut)
  self~assertTrue(YAML.deepEqual(doc, doc2), "directives full roundtrip")

  /* 35.9 Multi-doc round-trip with directives */
  p9 = .Yaml~new
  yaml = "%YAML 1.2"   || "0A"x || -
         "---"          || "0A"x || -
         "first: 1"     || "0A"x || -
         "..."          || "0A"x || -
         "%YAML 1.1"    || "0A"x || -
         "---"          || "0A"x || -
         "second: 2"
  docs = p9~parseAll(yaml)
  dirMap = p9~directivesMap
  dir1 = dirMap~at(docs[1])
  dir2 = dirMap~at(docs[2])
  self~assertEquals("1.2", dir1["yamlVersion"], "multi-doc dir1 version")
  self~assertEquals("1.1", dir2["yamlVersion"], "multi-doc dir2 version")

  /* 35.10 No directives: directivesMap entry is absent */
  p10 = .Yaml~new
  doc = p10~parseString("a: 1")
  dir = p10~directivesMap~at(doc)
  self~assertTrue(dir == .nil, "no directives no entry")

  /* 35.11 XML round-trip with directives (XSD) */
  p11 = .Yaml~new
  yaml = "%YAML 1.2"                      || "0A"x || -
         "%TAG !e! tag:example.com,2000:" || "0A"x || -
         "---"                             || "0A"x || -
         "name: test"
  doc = p11~parseString(yaml)
  dirMap = p11~directivesMap
  xml = .Yaml~yamlToXml(doc, "xsd", .nil, dirMap)
  self~assertTrue(xml~pos('yaml-version="1.2"') > 0, "XML has yaml-version")
  self~assertTrue(xml~pos('tag-directive') > 0, "XML has tag-directive")
  self~assertTrue(xml~pos('handle="!e!"') > 0, "XML has handle attr")
  self~assertTrue(xml~pos('prefix="tag:example.com,2000:"') > 0, "XML has prefix attr")
  p11b = .Yaml~new
  doc2 = p11b~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "XML directive roundtrip data")
  dir2 = p11b~directivesMap~at(doc2)
  self~assertTrue(dir2 \== .nil, "XML directive roundtrip has directives")
  self~assertEquals("1.2", dir2["yamlVersion"], "XML directive roundtrip version")
  th2 = dir2["tagHandles"]
  self~assertEquals("tag:example.com,2000:", th2["!e!"], "XML directive roundtrip handle")

  /* 35.12 XML round-trip with directives (DTD) */
  p12 = .Yaml~new
  yaml = "%YAML 1.2"                   || "0A"x || -
         "%TAG !! tag:yaml.org,2002:" || "0A"x || -
         "---"                          || "0A"x || -
         "val: 42"
  doc = p12~parseString(yaml)
  dirMap = p12~directivesMap
  xml = .Yaml~yamlToXml(doc, "dtd", .nil, dirMap)
  self~assertTrue(xml~pos('yaml-version="1.2"') > 0, "DTD XML has yaml-version")
  self~assertTrue(xml~pos('tag-directive') > 0, "DTD XML has tag-directive")
  p12b = .Yaml~new
  doc2 = p12b~parseXml(xml)
  self~assertTrue(YAML.deepEqual(doc, doc2), "DTD XML directive roundtrip data")

  /* 35.13 P9: tags stored as shorthand with preserveTags */
  p13 = .Yaml~new(.true, .true)
  yaml = "%TAG !e! tag:example.com,2000:" || "0A"x || -
         "---"                            || "0A"x || -
         "name: !e!person Alice"
  doc = p13~parseString(yaml)
  val = doc["name"]
  self~assertTrue(val~isA(.YamlTagged), "P9 tag is YamlTagged")
  self~assertEquals("!e!person", val~tag, "P9 !e! stored as shorthand")
  self~assertEquals("Alice", val~value, "P9 tag value preserved")

  /* 35.14 P9: !! stored as shorthand */
  p14 = .Yaml~new(.true, .true)
  yaml = "%TAG !! tag:yaml.org,2002:" || "0A"x || -
         "---"                        || "0A"x || -
         "count: !!int 42"
  doc = p14~parseString(yaml)
  val = doc["count"]
  self~assertTrue(val~isA(.YamlTagged), "P9 !!int is YamlTagged")
  self~assertEquals("!!int", val~tag, "P9 !! stored as shorthand")

  /* 35.15 P9: primary handle ! stored as shorthand */
  p15 = .Yaml~new(.true, .true)
  yaml = "%TAG ! tag:custom.org/" || "0A"x || -
         "---"                    || "0A"x || -
         "item: !widget Sprocket"
  doc = p15~parseString(yaml)
  val = doc["item"]
  self~assertTrue(val~isA(.YamlTagged), "P9 !widget is YamlTagged")
  self~assertEquals("!widget", val~tag, "P9 ! stored as shorthand")
  self~assertEquals("Sprocket", val~value, "P9 ! handle value")

  /* 35.16 P9: no %TAG means no resolution */
  p16 = .Yaml~new(.true, .true)
  yaml = "name: !custom Alice"
  doc = p16~parseString(yaml)
  val = doc["name"]
  self~assertEquals("!custom", val~tag, "P9 no TAG no resolution")

  /* 35.17 P9: lazy resolution via resolveTagHandle */
  p17 = .Yaml~new(.true, .true)
  yaml = "%TAG !e! tag:example.com,2000:" || "0A"x || -
         "%TAG !! tag:yaml.org,2002:"    || "0A"x || -
         "---"                            || "0A"x || -
         "name: !e!person Alice"          || "0A"x || -
         "count: !!int 42"
  doc = p17~parseString(yaml)
  dir = p17~directivesMap~at(doc)
  th = dir["tagHandles"]
  resolved = p17~resolveTagHandle("!e!person", th)
  self~assertEquals("!<tag:example.com,2000:person>", resolved, "P9 lazy !e! resolved")
  resolved = p17~resolveTagHandle("!!int", th)
  self~assertEquals("!<tag:yaml.org,2002:int>", resolved, "P9 lazy !! resolved")
  resolved = p17~resolveTagHandle("!custom", th)
  self~assertEquals("!custom", resolved, "P9 lazy no match unchanged")

/*------------------------------------------------------------------------*/
/* 36. Tags (!!str, !!int, !custom)                                       */
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
