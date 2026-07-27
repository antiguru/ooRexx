/* extracted from ADDRESS::test_environment_unix */
::routine main public
  if .ooRexxUnit.OSName == "WINDOWS" then
    return

  -- on a Unix platform Rexx defines a bunch of environments
  -- we check if these environments call the appropriate shell (if installed)
  -- if the shell for a given environment isn't installed, we expect a FAILURE
  -- if the shell is available, we expect "echo $0" to return the shell name
  -- (we don't test 'csh' and 'path', as they don't support "echo $0")
  environments = "1::" || address(), "1:sh", -
    "0:bsh", "0:bash", "0:ksh", "0:tcsh", "0:zsh", "1:command:sh", "1:system:sh"

  do env over environments
    parse var env required ":" shell ":" echo
    if echo = "" then
      echo = shell
    if shell == address() then
      required = .true
    failed = .false
    address value shell with output stem out.
    "echo $0"
    failed = .rs == -1

    -- we do not expect all environments to find their shell actually installed
    -- address(), "", "sh", "command" and "path" should work at all times,
    -- all others (if not installed) will raise FAILURE, which is a valid outcome
    if failed, \required then
      iterate                          -- FAILURE is ok, shell not installed

    -- environments deemed as required, should always work
    if failed then
      self~assertFalse(failed, "environment '"shell"' failed")

    -- echo $0 output should end with the shell name, e.g. /bin/sh
    self~assertTrue(out.1~endsWith("/" || echo), "environment '"shell"' expected to return ../" || echo "for echo $0")
  end

-- tests for "address-with" forms
/*

ADDRESS (environment command | VALUE env_expression) WITH - fragment

WITH - fragment allows up to three INPUT/OUTPUT/ERROR combinations of:
  WITH INPUT                       NORMAL
  WITH INPUT                       STEM stem / STREAM stream / USING ( object )
  WITH OUTPUT/ERROR                NORMAL
  WITH OUTPUT/ERROR                STEM stem / STREAM stream / USING ( object )
  WITH OUTPUT/ERROR REPLACE/APPEND STEM stem / STREAM stream / USING ( object )

address ::= 'ADDRESS' (  |
                         ( environment expression? |
                           'VALUE'? env_expression ) ( WITH - fragment )? )
address_with ::=      'WITH' ( 'INPUT' ( 'NORMAL' | 'STEM' stem | 'STREAM' stream | 'USING' expr ) |
                             ( 'OUTPUT' | 'ERROR' ) ( 'NORMAL' |
                                                    ( | 'REPLACE' | 'APPEND' ) ( 'STEM' stem | 'STREAM' stream | 'USING' expr ) ) )+

*/
-- creates a file by writing a few lines
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
