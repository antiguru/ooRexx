/* extracted from Stream::test_bug_1917 */
::routine main public
  f = .TemporaryTestFile~new(, "test_bug_1917")
  f~create(("first line", "second line", "third line"))
  s = .Stream~new(f)

  call read , 2
  call read "linein"
  call read "2 line"
  call read "=2 line"
  call read "+1 line"
  call read ("3 line", "-1 line")
  call read ("3 line", "-2 line", "+1 line")
  call read ("+99 line", "2 line")
  call read ("5 char", "2 line")
  call read ("linein", "5 char", "2 line")
  call read ("5 char", "linein 2", "2 line")
  call read ("5 char", "linein", "2 line")
  call read ("5 char", "linein"), 2
  exit

  read:
  s~open
  do c over arg(1)
    select
      when c = "linein" then s~linein
      when c~abbrev("linein") then s~linein(c~subwords(2))
      otherwise s~seek(c)
    end
  end
  if arg(2, "exists") then
    self~assertSame("second line", s~linein(arg(2)), arg(1)~makeArray~~append("("arg(2)")")~toString(, ", "))
  else
    self~assertSame("second line", s~linein, arg(1)~makeArray~toString(, ", "))
  s~close
  return


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
