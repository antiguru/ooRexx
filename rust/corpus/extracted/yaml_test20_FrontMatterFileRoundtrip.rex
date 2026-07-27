/* extracted from yaml::test20_FrontMatterFileRoundtrip */
::routine main public
  expose parser thisLocation

  fmData = .table~new
  fmData["title"] = "Advanced ooRexx"
  fmData["lang"] = "en-GB"
  authors = .array~new
  a1 = .table~new; a1["name"] = "John Smith"; a1["affiliation"] = "ACME Corp"
  a2 = .table~new; a2["name"] = "Jane Doe";   a2["affiliation"] = "Widgets Inc"
  authors~append(a1); authors~append(a2)
  fmData["author"] = authors
  fmData["keywords"] = .array~of("ooRexx", "YAML", "parser")

  fmFile = thisLocation"test20_frontmatter.yaml"
  .Yaml~toYamlFMFile(fmData, fmFile)

  fmDoc = parser~parseFrontMatterFile(fmFile)
  self~assertTrue(fmDoc~isA(.table), "fm roundtrip type")
  self~assertTrue(YAML.deepEqual(fmData, fmDoc), "fm roundtrip equal")
  self~assertEquals("Advanced ooRexx", fmDoc["title"], "fm roundtrip title")
  self~assertEquals("en-GB", fmDoc["lang"], "fm roundtrip lang")
  self~assertEquals(2, fmDoc["author"]~items, "fm roundtrip authors")
  self~assertEquals("John Smith", fmDoc["author"][1]["name"], "fm roundtrip author1")
  self~assertEquals(3, fmDoc["keywords"]~items, "fm roundtrip kw")

  .Yaml~toYamlFMFile(fmDoc, fmFile)
  fmDoc2 = parser~parseFrontMatterFile(fmFile)
  self~assertTrue(YAML.deepEqual(fmDoc, fmDoc2), "fm roundtrip stable")

  -- cleanup
  call SysFileDelete fmFile

/*------------------------------------------------------------------------*/
/* 21. XML round-trip via XSD                                             */
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
