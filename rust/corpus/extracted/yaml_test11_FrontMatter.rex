/* extracted from yaml::test11_FrontMatter */
::routine main public
  expose parser

  md = "---"                      || "0A"x || -
       "title: My Post"           || "0A"x || -
       "date: 2025-01-15"         || "0A"x || -
       "tags: [yaml, rexx]"       || "0A"x || -
       "---"                      || "0A"x || -
       ""                         || "0A"x || -
       "# This is the body"       || "0A"x || -
       "Not parsed as YAML."

  fm = parser~parseFrontMatter(md)
  self~assertEquals("My Post", fm["title"], "fm title")
  self~assertEquals("2025-01-15", fm["date"], "fm date")
  tags = fm["tags"]
  self~assertEquals(2, tags~items, "fm tag count")
  self~assertEquals("yaml", tags[1], "fm tag 1")

/*------------------------------------------------------------------------*/
/* 12. Comments                                                           */
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
