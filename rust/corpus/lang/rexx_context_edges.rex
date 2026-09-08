/* The spellings of `.CONTEXT`'s rows that `rexx_context.rex` does not reach,
   enumerated by naming the class each row's defects would share rather than by
   adding cases to the program that already passed.

   WHICH WRONG ANSWER EACH LINE PRINTS.

   E1: `~variables` holds a name only while it HAS a value, so a DROPped one
   goes. An engine listing every name the plan reserved prints `1 1`.

   E2: a PROCEDURE'd label has a pool of its own, where the label in
   `rexx_context.rex` shares its caller's -- the same method over two
   activations, answering opposite ways, which is what says the reader is
   reading the activation and not the program.

   E3/E4: one ::ROUTINE body reached as a function and by CALL. `~name` is the
   invocation name, which is the same word both ways here; an engine answering
   the DECLARING spelling would still print RTNFN, so this pair pins the route
   rather than the case -- `rexx_context.rex`'s `call 'lower'` is not written
   because a quoted lower-case call is a second question.

   E5/E6: an INHERITED method. The activation's scope is BASE and its receiver
   is a SUB, so an engine reading `~executable` off the receiver's class rather
   than the defining one answers for the wrong dictionary, and one reading
   `~target` off the scope prints `a BASE`.

   E7: `~line` inside two nested DO blocks. The clause is resolved on
   `run_bounded`'s own local program counter, so an engine reading the
   activation's `pc` prints the outer DO's line instead. */

aa = 'one'
bb = 'two'
drop bb
d = .context~variables
say 'E1 [' || d~hasIndex('AA') || '][' || d~hasIndex('BB') || ']'
call proc_label
say 'E3 [' || rtnfn() || ']'
call rtnfn
say 'E4 [' || result || ']'
o = .sub~new
z = o~m
do i = 1 to 1
  do j = 1 to 1
    say 'E7 [' || .context~line || ']'
  end
end
exit 0

proc_label: procedure
  say 'E2 [' || .context~variables~hasIndex('AA') || '][' || .context~name || ']'
  return

::routine rtnfn
  return .context~name

::class base
::method m
  say 'E5 [' || .context~name || '][' || .context~executable~class~id || ']'
  say 'E6 [' || .context~stackFrames[1]~target~string || '][' || .context~stackFrames[1]~type || ']'
  return 0

::class sub subclass base
