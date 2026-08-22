/* provide.xml `reqstr` names "arguments to built-in functions" wholesale, and
 * what follows from where the oracle converts is witnessed here.
 *
 * The order is position order, whatever order the builtin reads its arguments
 * in: SUBSTR reads its length and its pad before its subject, and the say
 * lines from its makeStrings still come out 1, 2, 3.
 *
 * SUBSTR fetches its pad through `optional_pad` unconditionally, so the pad is
 * converted whether or not the length reaches past the subject -- the pad's
 * own line prints for both calls below and only the second one pads with it.
 * **That is a property of this fetch and not of builtin arguments in
 * general**: a position the oracle fetches with `stack->peek` is never
 * converted, and `required_string_builtin_raw_argument.rex` is the one such
 * position in the whole set.
 *
 * A bad argument names the *object* and not the conversion, because the
 * oracle's error path is handed the value the expression produced. The
 * message shapes the raises below cover are a whole number, a pad and a
 * number. The pad is the exception that fixes the rule: padArgument converts
 * first and quotes the conversion, so its own line reads the makeString
 * answer.
 *
 * Phase 5a Task 14.
 */

/* The counter is set before the trap is armed, for the reason
   required_string_operator_argument.rex gives: a build where one of the
   answering clauses below raises instead must fail this program rather than
   loop in it. */
n = 0
signal on syntax name trapped

say substr(.subject, .from, .count, .pad)
say substr('abc', 1, 9, .pad)
say length(.subject)
say word(.subject, .from)
say translate(.subject, .pad, 'b')

next:
n = n + 1
select
  when n = 1 then say substr(.subject, .notanumber)
  when n = 2 then say substr('abc', 1, 9, .notapad)
  when n = 3 then say max(1, .notanumber)
  when n = 4 then say copies(.subject, .notanumber)
  otherwise signal done
end
say n 'answered'
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
say 'done'
exit 0

::class subject
::method makeString class
  say '1 subject'
  return 'abcdefgh'

::class from
::method makeString class
  say '2 from'
  return 2

::class count
::method makeString class
  say '3 count'
  return 3

::class pad
::method makeString class
  say '4 pad'
  return '-'

::class notanumber
::method makeString class
  return 'not a number'

::class notapad
::method makeString class
  return 'two'
