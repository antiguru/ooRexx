/* extracted from rxQueue::test_rxqueue_bug_1471 */
::routine main public

  -- a rather obscure bug, seen on Windows, maybe starting with MSVC 2015
  -- when RXQUEUE reads piped(*) stdin, a Cr-Lf sequence that happens to be
  -- exactly at position 8193 will simply get lost
  -- (*) this only happens if stdin is *piped* to rxqueue,
  --   e. g. "type abc.txt | rxqueue"
  --   not when read directly, e. g. "rxqueue < abc.txt"

  if .ooRexxUnit.OSName \== "WINDOWS" then
    return

  -- we create a file with 500 lines, 15 chars each
  -- the Cr-Lf sequence of line 482 will be exactly at position 8193:
  -- 481 lines of length 17 (including Cr-Lf), plus 15 chars = 8192
  -- the Cr-Lf sequence following line 482 gets lost due to [bugs:#1471]
  a = .Array~new(500)~fill("0123456789ABCDE")

  bug1471 = .TemporaryTestFile~new(self, "rxqueue_bug_1471")
  bug1471~create(a)
  .stdque~empty
  "type" bug1471~quotedName "| rxqueue"
  self~assertEquals(500, queued())
  self~assertEquals(a~makeString("c"), .stdque~makeArray~makeString("c"))
  self~assertEquals(0, queued())

  bug1471~delete


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
