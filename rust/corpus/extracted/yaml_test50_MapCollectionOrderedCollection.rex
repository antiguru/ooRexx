/* extracted from yaml::test50_MapCollectionOrderedCollection */
::routine main public

  p = .Yaml~new

  /* StringTable as map input */
  st = .StringTable~new
  st["name"] = "Alice"
  st["age"]  = "30"
  yamlSt = .Yaml~toYaml(st)
  docSt  = p~parseString(yamlSt)
  self~assertEquals("Alice", docSt["name"], "StringTable toYaml > name")
  self~assertEquals("30", docSt["age"], "StringTable toYaml > age")

  /* Directory as map input */
  dir = .Directory~new
  dir["host"] = "localhost"
  dir["port"] = "8080"
  yamlDir = .Yaml~toYaml(dir)
  docDir  = p~parseString(yamlDir)
  self~assertEquals("localhost", docDir["host"], "Directory toYaml > host")
  self~assertEquals("8080", docDir["port"], "Directory toYaml > port")

  /* Queue as sequence input (via wrapper map) */
  q = .Queue~new
  q~queue("first")
  q~queue("second")
  q~queue("third")
  wrapper = .Table~new
  wrapper["items"] = q
  yamlQ = .Yaml~toYaml(wrapper)
  docQ  = p~parseString(yamlQ)
  self~assertEquals(3, docQ["items"]~items, "Queue toYaml > count")
  self~assertEquals("first", docQ["items"][1], "Queue toYaml > item 1")
  self~assertEquals("third", docQ["items"][3], "Queue toYaml > item 3")

  /* List as sequence input (via wrapper map) */
  li = .List~new
  li~insert("alpha")
  li~insert("beta", li~last)
  li~insert("gamma", li~last)
  wrapper2 = .Table~new
  wrapper2["data"] = li
  yamlLi = .Yaml~toYaml(wrapper2)
  docLi  = p~parseString(yamlLi)
  self~assertEquals(3, docLi["data"]~items, "List toYaml > count")
  self~assertEquals("alpha", docLi["data"][1], "List toYaml > item 1")
  self~assertEquals("gamma", docLi["data"][3], "List toYaml > item 3")

  /* yamlToXml with StringTable */
  xmlSt = .Yaml~yamlToXml(st, "xsd")
  self~assertTrue(xmlSt~pos("<mapping>") > 0, "StringTable yamlToXml > has mapping")
  self~assertTrue(xmlSt~pos("Alice") > 0, "StringTable yamlToXml > has Alice")

  /* yamlToXml with Queue (via wrapper) */
  xmlQ = .Yaml~yamlToXml(wrapper, "xsd")
  self~assertTrue(xmlQ~pos("<sequence>") > 0, "Queue yamlToXml > has sequence")
  self~assertTrue(xmlQ~pos("first") > 0, "Queue yamlToXml > has first")
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
