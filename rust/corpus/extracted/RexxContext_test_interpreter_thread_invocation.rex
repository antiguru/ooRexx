/* extracted from RexxContext::test_interpreter_thread_invocation */
::routine main public

  self~assertTrue(Datatype(.context~interpreter,"Whole number"), "interpreter id must be a whole number, got:" .context~interpreter)
  self~assertTrue(Datatype(.context~thread     ,"Whole number"), "thread id must be a whole number, got:" .context~thread)
  self~assertTrue(Datatype(.context~invocation ,"Whole number"), "invocation id must be a whole number, got:" .context~invocation)

  currThread    =.context~thread
  currInvocation=.context~invocation

  r=.routine~new("test_new_context_values", .resources~context_thread_invocation)
  st=r~call

/* --
say "currThread    ="currThread
say "currInvocation="currInvocation
say

say "st~items:" st~items
do counter c idx over st~allindexes~sort
   say c~right(2)":" idx"=["st[idx]']'
end
-- */
  self~assertTrue(currThread=st~01_prologthread)
  self~assertTrue(currInvocation<st~02_prologinvocation)

    -- assert threads
  self~assertTrue(st~01_prologthread =  st~11_m3reply_before_thread, st~01_prologthread "=" st~11_m3reply_before_thread)
  self~assertTrue(st~01_prologthread <> st~13_m3reply_after_thread,  st~01_prologthread "<>" st~13_m3reply_after_thread)
  self~assertTrue(st~11_m3reply_before_thread <> st~13_m3reply_after_thread,  st~11_m3reply_before_thread "<>" st~13_m3reply_after_thread)

  self~assertTrue(st~01_prologthread <> st~21_m2start_thread, st~01_prologthread "<>" st~21_m2start_thread)
  self~assertTrue(st~01_prologthread =  st~31_m1_thread     , st~01_prologthread "="  st~31_m1_thread     )
  self~assertTrue(st~01_prologthread =  st~41_routine1thread, st~01_prologthread "="  st~41_routine1thread)

   -- assert invocatons
  self~assertTrue(st~02_prologInvocation < st~12_m3reply_before_invocation        , st~02_prologInvocation "<" st~12_m3reply_before_invocation        )
  self~assertTrue(st~02_prologInvocation < st~14_m3reply_after_invocation         , st~02_prologInvocation "<" st~14_m3reply_after_invocation         )
  self~assertTrue(st~12_m3reply_before_invocation = st~14_m3reply_after_invocation, st~12_m3reply_before_invocation "=" st~14_m3reply_after_invocation)

  self~assertTrue(st~02_prologInvocation < st~22_m2start_invocation               , st~02_prologInvocation "<" st~22_m2start_invocation)
  self~assertTrue(st~02_prologInvocation < st~32_m1_invocation                    , st~02_prologInvocation "<" st~32_m1_invocation     )
  self~assertTrue(st~02_prologInvocation < st~42_routine1invocation               , st~02_prologInvocation "<" st~42_routine1invocation)



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
