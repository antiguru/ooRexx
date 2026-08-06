/* PULL, PARSE PULL, PARSE LINEIN and the ARG instruction, against a queue
   this program fills itself and a console the corpus harness pins to
   /dev/null.

   IT ALSO READS THE QUEUE BACK. lang/push_queue.rex pins that PUSH and QUEUE
   evaluate, render and trace their expression, and says in its own header that
   it cannot pin which end of the queue a line landed on, because nothing that
   reads a line back existed when it was written. Block A is that reader, on
   stdout rather than only in a trace. crates/rexx-exec/tests/input_oracle.rs's
   `queue-round-trip` row is the same property with lines on the console as
   well, which no program here can have.

   WHICH WRONG ANSWER EACH BLOCK PRINTS.

   A: the interleaved order, read back. PUSH inserts at the head and QUEUE
   appends at the tail, so `push c` / `queue b` / `push a` stores a, c, b and
   three reads answer A, c, b -- the first upcased because PULL is PARSE UPPER
   PULL and the other two not. An engine that removed from the TAIL instead
   prints B, c, a. One that collapsed PUSH into QUEUE prints C, b, a. One that
   upcased at PUSH/QUEUE time rather than at PULL prints C for the second and
   third fields too; that is why the pushed literals are lower case and only
   one of the three reads is a bare PULL.

   B: the queue is now empty and the console is at end of input, which is the
   null string for every construct and for every repetition -- rc 0, no
   condition, no hang. An engine that raised a condition, or that answered the
   last line again, or that answered the derived variable name for an unset
   target, fails here. B4's PARSE LINEIN is the adjacent LINEIN read: it never
   consults the queue, which this block cannot show on its own (the queue is
   empty) and which C is here for.

   C: PARSE LINEIN does not consult the queue. One entry is queued and then
   PARSE LINEIN reads, so it must answer the null string and leave the entry
   alone for the PARSE PULL after it. An engine that let LINEIN fall back to
   the queue prints `[queued-not-for-linein][]` instead of `[][queued-not-for-
   linein]`.

   D: ARG template is PARSE UPPER ARG template, and PARSE ARG is not upper.
   The corpus harness passes no arguments, so both parse the null string and
   the case difference cannot show here -- what D pins is that both spellings
   RUN and assign the null string rather than leaving their targets unset, and
   that a bare ARG and a bare PARSE ARG are legal and do nothing observable.
   The case split and a non-empty argument need a command line, which no
   corpus program has; they are witnessed in crates/rexx-exec/tests/
   input_oracle.rs instead.

   E: the trace shape, under TRACE R and then TRACE I, because a target's own
   value line is a CHOICE of prefix between the two modes and no stdout
   witness can reach either. The `>K>` line is what the two modes agree on,
   and for a bare PULL it carries the line BEFORE the upcase while the `>>>`
   after it carries the line after -- the two lines disagree on one
   instruction. ARG and PARSE ARG emit no `>K>` at all, which is the one
   source-shaped fact only a trace can show.

   This program prints no program path and reads no clock, so it obeys the
   corpus's determinism rule. Its console reads all answer the null string
   because the harness gives both interpreters an empty stdin; a program whose
   answers depended on the console's contents could not live here.               */

/* ---- A: the interleaved order, read back ------------------------------- */
push "c"
queue "b"
push "a"
pull n1
parse pull n2
parse pull n3
say "A [" || n1 || "][" || n2 || "][" || n3 || "]"

/* ---- B: an empty queue and an exhausted console ------------------------ */
pull n4
parse pull n5
parse pull n6
parse linein n7
say "B [" || n4 || "][" || n5 || "][" || n6 || "][" || n7 || "]"

/* ---- C: PARSE LINEIN never consults the queue -------------------------- */
queue "queued-not-for-linein"
parse linein n8
parse pull n9
say "C [" || n8 || "][" || n9 || "]"

/* ---- D: the ARG spellings ---------------------------------------------- */
arg n10 n11
parse arg n12 n13
say "D [" || n10 || "][" || n11 || "][" || n12 || "][" || n13 || "]"
arg
parse arg
say "D bare forms are legal"

/* ---- E: the trace shape, in both modes that show one ------------------- */
push "traced-lower"
trace r
pull n14
parse pull n15
parse linein n16
arg n17
parse arg n18
trace off
push "traced-lower"
trace i
pull n19
parse pull n20
parse linein n21
arg n22
parse arg n23
trace off
say "E done"
