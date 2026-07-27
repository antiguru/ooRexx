/* extracted from json::test_class_without_json_support_quotes */
::routine main public
  j=.json~new
  o=.class_without_json_support_quotes~new
--  say "o:" pp(o); say "j~toJson(o):" pp(j~toJson(o))
  -- escaping a slash (/) is optional in JSON, hence both, '"n_\"/\"_a"' and '"n_\"\/\"_a"' are valid results
  str= '"n_\"/\"_a"' '"n_\"\/\"_a"'
  pos=wordPos(j~toJson(o), str)
  self~assertTrue(pos>0, "'"j~toJson(o)"' is neither" '"n_\"/\"_a"' "nor" '"n_\"\/\"_a"')

  o=.class_without_json_support_quotes~new('o"o')
--  say "o:" pp(o); say "j~toJson(o):" pp(j~toJson(o))
  self~assertEquals('"o\"o"', j~toJson(o))



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
