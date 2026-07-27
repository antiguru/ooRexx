/* extracted from Method::test_new_file_compiled */
::routine main public
   -- delete tmpCodeFiles, then create them on this platform (compiled code is dependent on platform and bitness)
  files=.tmpCodeFiles   -- get the file names (already fully qualified)

  if .recreateTmpCodeFiles?=.true then -- indicate we need to recreate the files
  do
     call sysFileDelete files[1]
     .stream~new(files[1])~~open("write replace")~~lineout(.resources~testCode4newFile)~~close
     -- make sure we create the compiled version for this operating system
     call sysFileDelete files[2]
     address system "rexxc" '"'files[1]'"' '"'files[2]'"' "-s -e"   -- compile silently and encode
     call sysFileDelete files[3]
     address system "rexxc" '"'files[1]'"' '"'files[3]'"' "-s"      -- compile silently (binary version)
     .context~package~local~recreateTmpCodeFiles?=.false
  end

  do counter i fn over files     -- test source and encoded compiled
     m=.method~newFile(fn)       -- create method from array
     o=.object~enhanced(.directory~of(("m",m)))
     self~assertEquals(43,o~m)
     self~assertEquals(47,o~m(1,'a',3,4))
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
