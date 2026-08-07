/* The builtins that read interpreter state: ARG, CONDITION, DIGITS, FORM,
   FUZZ, GC, QUEUED, SOURCELINE and TRACE. ADDRESS has its own program
   (lang/address_env.rex), which the toggle and the callee-inheritance blocks
   there need to itself.

   WHICH WRONG ANSWER EACH BLOCK PRINTS.

   A: the NUMERIC trio before and after a NUMERIC instruction moves each one,
   and then across a call. The callee's own settings must not leak back, so
   A3 and A5 differ while A4 and A5 bracket them -- an engine holding one
   interpreter-wide setting prints the callee's 4 at A5 and passes every
   single-frame check.

   B: TRACE() reports the SETTING, not the behaviour. O, N, C, E and F all
   trace nothing at all, so an engine that stores only what tracing does
   cannot tell them apart and prints one letter five times. The initial
   setting -- before any TRACE instruction -- is N, and bare TRACE returns to
   it rather than to O. TRACE(new) answers the OLD setting and installs the
   new one, so B9 and B10 are two different letters from one clause pair.
   None of this block's settings echoes anything: every letter used here is
   one of the five silent ones, which is what keeps the block on stdout.

   C: ARG over a call with an interior omission. Position 2 is omitted and
   holds its place, so arg() is 3 rather than 2, and 'E'/'O' disagree at
   every position while 'N' repeats the bare form. A position past the end is
   not existing and is omitted, which is the pair that separates "out of
   range" from "empty". At the top level the harness passes no argument, so
   C1 is the zero case.

   F: the CALL ON half, which is the only place the other two answers for
   'I' and 'S' appear. A trap that fires as a CALL reports CALL, and while
   its handler runs the trap is DELAYED rather than off -- an engine that
   removes the trap for the handler's duration instead of holding it prints
   OFF at F1, and that is the whole of the difference the two mechanisms
   make. The condition does not survive the handler's return, which F3 is:
   an engine keeping one interpreter-wide condition object prints USER UC
   there.

   D: CONDITION from INSIDE a live handler, which is the only place it can be
   read at all -- outside one every option is the null string, so a stub
   returning '' passes any test written at the top level. D1 is the outside
   case for exactly that reason: it is the control, not the measurement.
   The bare form is CONDITION('I'), not ('C'). 'E' is the part of the code
   after the dot and only a SYNTAX condition has one. 'D' is the RAISE
   DESCRIPTION value and is empty for an interpreter-raised SYNTAX. 'S' is
   looked up LIVE in the trap table rather than stored beside 'I': re-arming
   the trap inside its own handler moves 'S' to ON and leaves 'I' at SIGNAL,
   and turning it off again moves 'S' back. An engine carrying one bit for
   both cannot print D4's SIGNAL ON at all. D6 is the reset: 'R' clears the
   condition and answers the null string, and everything after it is empty.

   E: ERRORTEXT is the major message only, so it never substitutes anything;
   a number in range with no catalogue entry is the null string rather than
   an error. SOURCELINE with no argument is this file's own line count and
   with one is that line verbatim -- E3 asks for line 1, which is the first
   line of this very comment, so an off-by-one prints a neighbour and a
   0-based index prints nothing or raises. E4 compares the count rather than
   printing it, so editing this file does not move the expected output.
   GC() is 0 and GC('Force') is 1. QUEUED counts what PUSH and QUEUE left
   behind and PULL removed.

   Determinism: no clock, no PID, no filesystem, no path. SOURCELINE reads
   this file's own text, which both interpreters are handed identically, and
   the queue is one this program fills itself. */

/* A */
say 'A1' digits() fuzz() form()
numeric digits 12
numeric fuzz 3
numeric form engineering
say 'A2' digits() fuzz() form()
numeric form
say 'A3' digits() fuzz() form()
call inner
say 'A5' digits() fuzz() form()

/* B */
say 'B1' trace()
trace off
say 'B2' trace()
trace
say 'B3' trace()
trace commands
say 'B4' trace()
trace errors
say 'B5' trace()
trace failure
say 'B6' trace()
trace normal
say 'B7' trace()
trace value 'O'
say 'B8' trace()
say 'B9' trace('N')
say 'B10' trace()
trace off

/* C */
say 'C1' arg() arg(1, 'E') arg(1, 'O')
call three 'p1',,'p3'

/* F */
call on user uc name caught
call raiser
say 'F3' '['condition()']['condition('C')']['condition('S')']'

/* D */
say 'D1' '['condition()']['condition('C')']['condition('S')']'
signal on syntax name handler
say 1/0
say 'D unreached'

/* E is reached from the handler, which exits. */

inner:
  numeric digits 4
  numeric fuzz 1
  say 'A4' digits() fuzz() form()
  return

caught:
  say 'F1' condition('I') condition('S')
  say 'F2' '['condition('C')']['condition('D')']['condition('E')']'
  return

raiser:
  raise user uc description 'from the raiser' return 1

three:
  say 'C2' arg()
  say 'C3' '['arg(1)']['arg(2)']['arg(3)']['arg(4)']'
  say 'C4' arg(1,'E') arg(2,'E') arg(3,'E') arg(4,'E')
  say 'C5' arg(1,'O') arg(2,'O') arg(3,'O') arg(4,'O')
  say 'C6' '['arg(2,'N')']['arg(3,'Normal')']'
  return

handler:
  say 'D2' '['condition()']['condition('C')']['condition('D')']['condition('E')']'
  say 'D3' condition('I') condition('S')
  signal on syntax name handler
  say 'D4' condition('I') condition('S')
  signal off syntax
  say 'D5' condition('I') condition('S')
  say 'D6' '['condition('R')']['condition('C')']['condition('I')']'

  /* E */
  say 'E1' errortext(5)
  say 'E2' '['errortext(0)']'
  say 'E3' '['sourceline(1)']'
  say 'E4' (sourceline() > 100)
  say 'E5' gc() gc('Force') gc('f')
  say 'E6' queued()
  queue 'first'
  push 'second'
  say 'E7' queued()
  pull line
  say 'E8' queued() line
  exit 0
