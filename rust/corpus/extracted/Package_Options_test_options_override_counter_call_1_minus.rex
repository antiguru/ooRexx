/* extracted from Package_Options::test_options_override_counter_call_1_minus */
::routine main public
  fn_base=.context~name
  fn1=fn_base"_a.rex"
  fn2=fn_base"_b.rex"
  fn3=fn_base"_c.rex"
  file1 = .TemporaryTestFile~new(.nil, fn1)
  file2 = .TemporaryTestFile~new(.nil, fn2)
  file3 = .TemporaryTestFile~new(.nil, fn3)
  fn1   =file1~fullName
  fn2   =file2~fullName
  fn3   =file3~fullName
  file1~create( ("call" quote(fn2), "rc2=result", "return digits()+rc2", "::options digits 10 fuzz 1") )
  file2~create( ("call" quote(fn3), "rc3=result", "return digits()+rc3", "::options           fuzz 2") )
  file3~create( (                      "return digits()   ", "::options           fuzz 3") )

   -- override from now on
  .package~defaultOptions("defineDefaultOptions", "::options digits 3 fuzz 0")
  .package~defaultOptions("count", -1)
   call (fn1)
   rc1=result
   self~assertEquals(16, rc1)
   self~assertEquals(-4, .package~defaultOptions("count"))


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
