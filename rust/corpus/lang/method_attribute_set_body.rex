/* A ::ATTRIBUTE SET written in Rexx is a method body like any other, and it
   is the accessor form whose dictionary key is not the directive's own name:
   the message a `.K~a = value` assignment sends is `A=`, so the entry the
   send resolves against carries an appended `=`. Both accessor forms carry a
   body of their own when they are written this way; only the generated
   accessors, which read an instance variable, do not.

   The `a` pair is the untraced half. S4 is the setter's own line and S3 is
   the getter's, so which body ran is visible on stdout: measured, an engine
   whose assignment resolves to the getter's entry instead prints S3 twice and
   S4 never.

   The `b` pair is why this program is in RAW_STDERR_COMPARISON. The >I> and
   <I< lines carry the RESOLVED name, so `"B"` from the getter against `"B="`
   from the setter is the only place a program can read the name itself rather
   than infer it from which body ran; an engine announcing the directive's own
   spelling prints `"B"` for both, on stderr alone, at rc 0. The bodies turn
   TRACE I on themselves rather than inheriting it, because trace does not
   cross into a method activation -- measured, a traced sending clause echoes
   its own >E>/>L>/>A> lines and no >I> at all.

   Neither accessor stores anything, and that is a property of these bodies
   rather than of the phase: a method activation's frame is its own, so a name
   assigned in the setter is not the name the getter reads. Neither body
   exposes anything, which is the whole difference from the generated pair
   corpus/lang/method_attribute_generated.rex runs -- that pair reaches the
   receiver's pool and this one reaches two frames.

   No abuttal anywhere: a symbol abutting a preceding string literal can be
   read as its hex or binary suffix, so every join below is an explicit ||. */

.K~a = 'given'
say 'S1 ['||.K~a||']'

trace i
.K~b = 'traced'
say .K~b
trace off
say 'S2 done'

::class K

::attribute a class get
  say 'S3 getter ran'
  return 'from the getter'

::attribute a class set
  use arg v
  say 'S4 setter saw ['||v||']'
  return

::attribute b class get
  trace i
  return 'from the traced getter'

::attribute b class set
  trace i
  use arg v
  say 'S5 setter saw ['||v||']'
  return
