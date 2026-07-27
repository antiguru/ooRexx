/* extracted from json::test_wikipedia_to_json_from_resources */
::routine main public
   -- fetch resource (returns string array) and get its string value
  wiki_json     = .resources~wikipedia.json          ~makeString
  wiki_min_json = .resources~wikipedia_minimized.json~makeString
  wiki_leg_json = .resources~wikipedia_legible.json  ~makeString

  j    = .json~new
  obj1 = j~fromJson(wiki_json)   -- returns a directory object
  min1 = j~toJson(obj1)          -- by default minimizes
  self~assertSame(min1,wiki_min_json)

  leg1 = j~toJson(obj1,.true)    -- get legible rendering
  self~assertSame(leg1,wiki_leg_json)

  obj2 = j~fromJson(min1)
  min2 = j~toJson(obj2)
  self~assertSame(min1,min2)
  leg2 = j~toJson(obj2,.true)
  self~assertSame(leg1,leg2)

  obj3 = j~fromJson(leg1)
  min3 = j~toJson(obj2)
  self~assertSame(min1,min3)
  leg3 = j~toJson(obj3,.true)
  self~assertSame(leg1,leg3)

  --- test via class
  obj1 = .json~fromJson(wiki_json)   -- returns a directory object
  min1 = .json~toJson(obj1)          -- by default minimizes
  self~assertSame(min1,wiki_min_json)

  leg1 = .json~toJson(obj1,.true)    -- get legible rendering
  self~assertSame(leg1,wiki_leg_json)

  obj2 = .json~fromJson(min1)
  min2 = .json~toJson(obj2)
  self~assertSame(min1,min2)
  leg2 = .json~toJson(obj2,.true)
  self~assertSame(leg1,leg2)

  obj3 = .json~fromJson(leg1)
  min3 = .json~toJson(obj2)
  self~assertSame(min1,min3)
  leg3 = .json~toJson(obj3,.true)
  self~assertSame(leg1,leg3)


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
