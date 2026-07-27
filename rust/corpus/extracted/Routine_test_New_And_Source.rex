/* extracted from Routine::test_New_And_Source */
::routine main public
  name="AHA"
  value="say 'hello'; say; say 'hallo';"
  r=.routine~new(name, value)
  self~assertSame(value, r~source~at(1))


  value2=.array~of("say 'hello';", "say;", "say 'hallo';")
  r2=.routine~new(name, value2)
  self~assertEquals(value2, r2~source)


  value3=.array~of("parse arg logLevel;" ,-
                   "a=b",- -- <-- *NO* semi-colon at end!
                   "-- some line comment " ,-
                   "say 'hello world' " )
  r3=.routine~new(name, value3)
  self~assertEquals(value3, r3~source)


  value4=.array~of("parse arg logLevel;" ,-
                   "a=b;",- -- <-- semi-colon at end!
                   "-- some line comment " ,-
                   "say 'hello world' " )
  r4=.routine~new(name, value4)
  src4=r4~source
  do i=1 to value4~items
     self~assertEquals(value4[i], src4[i])
  end

  self~assertEquals(value4, r4~source)

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
