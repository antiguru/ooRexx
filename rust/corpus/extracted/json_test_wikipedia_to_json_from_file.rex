/* extracted from json::test_wikipedia_to_json_from_file */
::routine main public
   -- fetch resource (returns string array) and get its string value
   -- resource names
  names ="wikipedia.json", "wikipedia_minimized.json", "wikipedia_legible.json"
  values  =.array~new
  inFiles =.array~new
  outFiles=.array~new
  do counter c n over names
     values[c]   = .resources~entry(n)         -- get resource's value
     inFiles[c]  = .TemporaryTestFile~new(.nil,n) ~~create(values[c])  -- create file, assign content
     outFiles[c] = .TemporaryTestFile~new(.nil,"json_tmpOut_"n)
  end

  do i=1 to 3     -- read from original json, minimized json, legible json
     obj = .json~fromJsonFile(inFiles[i])       -- read JSON file

     .json~toJsonFile(outFiles[2],obj,.false)
     s=.stream~new(outFiles[2])~~open("read")   -- open minimized for reading
     minimizedJson = s~charin(1,s~chars)
     s~close
     self~assertSame(minimizedJson,values[2])

     .json~toJsonFile(outFiles[3],obj)          -- legible=.false by default
     s=.stream~new(outFiles[3])~~open("read")   -- open minimized for reading
     minimizedJson = s~charin(1,s~chars)
     s~close
     self~assertSame(minimizedJson,values[2])

     .json~toJsonFile(outFiles[3],obj, .true)   -- legible=.true by default
     s=.stream~new(outFiles[3])~~open("read")   -- open minimized for reading
     legibleJson = s~charin(1,s~chars)
     s~close
     self~assertSame(legibleJson,values[3])
  end


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
