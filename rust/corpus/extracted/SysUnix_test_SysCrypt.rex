/* extracted from SysUnix::test_SysCrypt */
::routine main public
  -- on Linux all these algorithms are supported, on macOs just the first two,
  -- on OpenBSD only Blowfish, etc.

  -- name,               id/rounds,  salt bits,   hash chars
  algo = (,
    "DES",               "",                12,  0 +  64 / 6,, -- many platforms except OpenBSD
    "BSDi extended DES", "_0...",           24,  0 +  64 / 6,, -- other BSD's, MacOS
    "MD5",               "$1$",             24,  1 + 128 / 6,, -- many platforms
    "Blowfish flawed",   "$2a$12$",        128, -1 + 192 / 6,, -- OpenBSD
    "Blowfish recent",   "$2b$12$",        128, -1 + 192 / 6,, -- OpenBSD
    "NT-Hash",           "$3$",              0,  1 + 128 / 4,, -- FreeBSD
    "SHA-256",           "$5$rounds=9999$", 48,  1 + 256 / 6,, -- Linux, FreeBSD, Solaris
    "SHA-512",           "$6$rounds=9999$", 48,  1 + 512 / 6,, -- Linux, FreeBSD, Solaris
  )

  -- our tests expect that at least two of the above are supported
  supported = .Array~new

  pw = "password"
  saltString = .String~lower
  do a = 1 to algo~items by 4
    name = algo[a]
    id = algo[a + 1]
    salt = (algo[a + 2] / 6)~ceiling
    returned = algo[a + 3]~ceiling
    key = id || saltString~left(salt)
    r = SysCrypt(pw, key)
    if r = "" then -- algorithm not supported
      iterate

    -- openIndiana/Solaris/SunOS crypt() does not understand the BSDi format and
    -- will return a standard DES encryption with 13 chars instead.  Ignore this.
    if .Rexxinfo~platform == "SUNOS", id~startsWith("_") then
      iterate

    -- macOS/Darwin crypt() does not understand the $id$ format and will return
    -- a standard DES encryption with 13 chars instead.  We ignore this here.
    if .Rexxinfo~platform == "DARWIN", id~startsWith("$") then
      iterate

    -- NetBSD crypt() does not support the $2b$, $3$, $5$, and $6$ formats and
    -- returns the string "*0" or a $1$ encryption instead.  Ignore this.
    if .Rexxinfo~platform == "NETBSD", ("$2b", "$3$", "$5$", "$6$")~hasItem(id~left(3)) then
      iterate

    supported~append(name)

    -- the returned data should match key plus expected hash chars
    self~assertSame(key~length + returned, r~length, "crypt("pw"," key") should return" key~length + returned "characters, but returned" r~length .endofline r)

    -- the returned data should start with id plus salt, except for Blowfish,
    -- where last salt character may be a mismatch
    start = id~startsWith("$2")~?(key~left(key~length - 1), key)
    self~assertTrue(r~startsWith(start), name "crypt("pw"," key") should return a string starting with key, but returns" r)
  end
  self~assertTrue(supported~items >= 2, "crypt() is expected to support at least two algorithms, but only" supported~items "work(s):" supported~toString(, ", "))


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
