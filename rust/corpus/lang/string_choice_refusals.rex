/* `?`'s refusals, by error number.
 *
 * The trap can only print the number: CONDITION("O") answers a Directory and
 * CONDITION("D") is empty for SYNTAX, so the substituted text of an 88.901 is
 * out of reach from inside a handler. The two 88.901s here differ only in the
 * argument they name, so each is the untrapped tail of one program --
 * `true value` ends string_choice.rex and `false value` ends this one.
 *
 * rc 168.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  /* Both arguments are required, and the first is missed first. */
  when n = 1 then say n 'answered' '1'~'?'()
  when n = 2 then say n 'answered' '1'~'?'('y')
  when n = 3 then say n 'answered' '1'~'?'(, 'n')
  /* The receiver is read only once both arguments are there. */
  when n = 4 then say n 'answered' 'abc'~'?'('y', 'n')
  when n = 5 then say n 'answered' 'abc'~'?'()
  /* Logical is exactly "0" or "1", as text, with no coercion. */
  when n = 6 then say n 'answered' '01'~'?'('y', 'n')
  when n = 7 then say n 'answered' ''~'?'('y', 'n')
  when n = 8 then say n 'answered' ' 1 '~'?'('y', 'n')
  when n = 9 then say n 'answered' '1.0'~'?'('y', 'n')
  /* Two is the maximum, the way every other operator row's one is. */
  when n = 10 then say n 'answered' '1'~'?'(1, 2, 3)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say '1'~'?'('y')
