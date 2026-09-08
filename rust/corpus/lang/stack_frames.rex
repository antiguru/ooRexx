/* `RexxContext~stackFrames` and everything a `StackFrame` answers, over a
   stack carrying all four frame kinds this crate can produce.

   WHICH WRONG ANSWER EACH BLOCK PRINTS.

   B: the four frames come back INNERMOST FIRST, and each kind answers a
   different `~type`. An engine walking the stack the other way prints B1's
   fields on B4's line. `~line` is the clause each frame is CURRENTLY
   executing, not the line it was entered at, so B3's line is the `z = ...`
   clause inside the label and not the `call outer` above it; an engine
   reporting the entry line prints B3's third field as the label's own line.
   `~target` is the receiver and appears on the METHOD frame alone.

   `~invocation` is minted on first ASK off a counter, and nothing here asked
   before, so the four ids follow the walk: 1, 2, 3, 4. That is NOT the depth
   -- `rexx_context.rex` asks at the top level first and its PROGRAM frame
   reads 1 there while its inner frames read more.

   C: `~traceLine` is the frame's own clause as TRACE would echo it, with the
   line number in a six-wide field and the runtime indent after the marker.
   C3's clause is inside a called label, whose activation indent is the
   caller's plus two, so it carries two spaces where C1, C2 and C4 carry none.
   An engine rendering the source line's own leading blanks instead prints
   C1's clause indented and C3's at the wrong width.

   X: `~string` and `~makeString` are bound to the same function as
   `~traceLine` (Setup.cpp), so all three are one string; an engine
   implementing them separately can print `[0]` in either of the last two
   fields. `~context` is a RexxContext on EVERY frame, including the three
   that never named `.context` -- building a frame creates one.

   D: a StackFrame OUTLIVES its activation and a RexxContext does not. D1-D3
   are read after `snapshot` returned and still answer; the last clause reads
   the matching context and is left untrapped, so the 98.981 report on stderr
   is the other half of the pair. An engine that made the frame a live handle
   raises on D1 instead.

   The PROGRAM frame's `~name` is the program's own path, so B4 compares it
   against `parse source`'s third word rather than printing it. */

parse source . . me
call outer
pair = snapshot('p1', 'p2')
ctx = pair[1]
frm = pair[2]
say 'D1 [' || frm~name || '][' || frm~line || '][' || frm~type || ']'
say 'D2 [' || frm~traceLine || ']'
say 'D3 [' || frm~arguments~items || '][' || frm~arguments[1] || '][' || frm~target~string || ']'
say 'D4 [' || ctx~class~id || '][' || ctx~objectName || ']'
say 'D5 [' || ctx~name || ']'
say 'D5 [not reached]'
exit 0

outer:
  z = .kk~new~m(7, 8)
  return

::class kk
::method m
  use arg p, q
  call inner 'aa'
  return 0

::routine inner
  parse source . . me
  f = .context~stackFrames
  say 'A1 [' || f~class~id || '][' || f~items || ']'
  do i = 1 to f~items
     fr = f[i]
     nm = fr~name
     if i = f~items then nm = 'path=' || (nm == me)
     say 'B' || i || ' [' || fr~type || '][' || nm || '][' || fr~line || '][' || fr~invocation || '][' || fr~target~string || ']'
     say 'C' || i || ' [' || fr~traceLine || ']'
     say 'X' || i || ' [' || fr~arguments~class~id || '][' || fr~arguments~items || '][' || fr~context~class~id || '][' || (fr~string == fr~traceLine) || '][' || (fr~makeString == fr~traceLine) || ']'
  end
  say 'A2 [' || (f[1]~context~identityHash = .context~identityHash) || ']'
  return

::routine snapshot
  c = .context
  f = c~stackFrames[1]
  return .array~of(c, f)
