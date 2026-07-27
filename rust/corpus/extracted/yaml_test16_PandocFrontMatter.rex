/* extracted from yaml::test16_PandocFrontMatter */
::routine main public
  expose parser

  md = "---"                                || "0A"x || -
       "title: 'Advanced ooRexx'"            || "0A"x || -
       "author:"                             || "0A"x || -
       "  - name: John Smith"                || "0A"x || -
       "    affiliation: ACME Corp"          || "0A"x || -
       "  - name: Jane Doe"                  || "0A"x || -
       "    affiliation: Widgets Inc"        || "0A"x || -
       "abstract: |"                         || "0A"x || -
       "  This paper presents a new"         || "0A"x || -
       "  approach to YAML parsing."         || "0A"x || -
       "keywords: [ooRexx, YAML, parser]"    || "0A"x || -
       "lang: en-GB"                         || "0A"x || -
       "bibliography: refs.bib"              || "0A"x || -
       "---"                                 || "0A"x || -
       ""                                    || "0A"x || -
       "# Introduction"                      || "0A"x || -
       "This is the body."

  fm = parser~parseFrontMatter(md)
  self~assertEquals("Advanced ooRexx", fm["title"], "pandoc title")
  authors = fm["author"]
  self~assertEquals(2, authors~items, "pandoc authors")
  self~assertEquals("John Smith", authors[1]["name"], "pandoc author1")
  self~assertEquals("Widgets Inc", authors[2]["affiliation"], "pandoc affil2")
  self~assertTrue(fm["abstract"]~pos("approach") > 0, "pandoc abstract")
  kw = fm["keywords"]
  self~assertEquals(3, kw~items, "pandoc keywords")
  self~assertEquals("en-GB", fm["lang"], "pandoc lang")
  self~assertEquals("refs.bib", fm["bibliography"], "pandoc bib")

/*------------------------------------------------------------------------*/
/* 17. Comprehensive serialisation round-trip                             */
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
