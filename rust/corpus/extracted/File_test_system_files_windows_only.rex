/* extracted from File::test_system_files_windows_only */
::routine main public
  if \.RexxInfo~platform~caselessStartsWith("WINDOWS") then
    return

  numeric digits 18 -- we expect very large file sizes
  do system over "hiberfil.sys", "swapfile.sys", "pagefile.sys"
    -- does the file exist?
    call SysFileTree SysBootDrive() || "\" || system, f., "fo"
    if f.0 = 0 then
      iterate
    name = f.1
    file = .File~new(name)
    self~assertTrue(file~exists, name "exists")
    self~assertTrue(file~isFile, name "isFile")
    self~assertFalse(file~isDirectory, name "isDirectory")
    self~assertTrue(file~isHidden, name "isHidden")
    self~assertTrue(file~length > 0, name "length")

    self~assertSame(file~lastModified~isoDate~left(19)~changeStr("T", " "), SysGetFileDateTime(name, "write"), "lastModified" name)
    self~assertSame(file~lastAccessed~isoDate~left(19)~changeStr("T", " "), SysGetFileDateTime(name, "access"), "lastAccessed" name)

    self~assertTrue(SysFileExists(name), "SysFileExists" name)
    self~assertTrue(SysIsFile(name), "SysIsFile" name)
    self~assertFalse(SysIsFileDirectory(name), "SysIsFileDirectory" name)
  end

   -- tests for RFE #872 (https://sourceforge.net/p/oorexx/feature-requests/872/)
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
