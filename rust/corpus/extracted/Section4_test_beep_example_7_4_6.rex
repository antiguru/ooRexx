/* extracted from Section4::test_beep_example_7_4_6 */
::routine main public
   -- Note: this only works on some Windows implementations. It is ignored
   --       on Windows 95 and all *nix versions.
   /* C scale */
   note.1 = 262    /* middle C */
   note.2 = 294    /*    D     */
   note.3 = 330    /*    E     */
   note.4 = 349    /*    F     */
   note.5 = 392    /*    G     */
   note.6 = 440    /*    A     */
   note.7 = 494    /*    B     */
   note.8 = 523    /*    C     */
   do i = 1 to 8
      call beep note.i, 250    /* hold each note for one-quarter second */
      ret = RESULT
      self~assertSame("", ret, "Loop:" i "Calling beep should produce a result in RESULT")
      end
   self~assertTrue(i == (8 + 1))


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
