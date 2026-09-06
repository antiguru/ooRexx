/* The operator messages' refusals.
 *
 * A missing operand is 93.903 for every binary operator; the arithmetic ones
 * raise 41.1 on a non-numeric operand and the logical ones 34.901 on an
 * operand that is not exactly "0" or "1". The prefix \ takes no operand at
 * all, so it has no missing-argument case and raises 34.901 on its receiver.
 *
 * An arithmetic operator sent bare converts its receiver before it misses its
 * operand, so which of the two errors comes out depends on the receiver, not
 * on the operator. `'abc'~"*"()` is 41.1 and `'12'~"*"()` is 93.903 -- both
 * halves are here, because either alone is satisfied by the wrong order. `+`
 * and `-` bare are the prefix forms and answer, so they appear here too.
 *
 * The six strict-ordering operators are NOT sent bare here: `'abc'~"<<"`
 * segfaults the oracle, which corpus/oracle-crashes.txt carries and
 * ORACLE_CRASHING_SENDS exempts. They are sent with an operand instead.
 *
 * The final send is untrapped, and it is an operator sent as a *message*, so
 * the `Compiled method "+" with scope "String".` line an operator message
 * carries and an operator expression does not is compared as bytes. rc 215.
 */

signal on syntax name trapped
s = 'abc'
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' s~'+'
  when n = 2 then say n 'answered' s~'='
  when n = 3 then say n 'answered' s~'||'
  when n = 4 then say n 'answered' s~'&'
  when n = 5 then say n 'answered' s~'+'(1)
  when n = 6 then say n 'answered' s~'-'(1)
  when n = 7 then say n 'answered' s~'*'(1)
  when n = 8 then say n 'answered' s~'**'(2)
  when n = 9 then say n 'answered' s~'/'(1)
  when n = 10 then say n 'answered' s~'%'(1)
  when n = 11 then say n 'answered' s~'//'(1)
  when n = 12 then say n 'answered' '1'~'/'('0')
  when n = 13 then say n 'answered' s~'&'('1')
  when n = 14 then say n 'answered' s~'|'('1')
  when n = 15 then say n 'answered' s~'&&'('1')
  when n = 16 then say n 'answered' s~'\'()
  when n = 17 then say n 'answered' '1'~'\'(2)
  /* The strict-ordering six, with an operand rather than bare. */
  when n = 18 then say n 'answered' s~'<<'('b')
  when n = 19 then say n 'answered' s~'<<='('b')
  when n = 20 then say n 'answered' s~'>>'('b')
  when n = 21 then say n 'answered' s~'>>='('b')
  when n = 22 then say n 'answered' s~'\<<'('b')
  when n = 23 then say n 'answered' s~'\>>'('b')
  /* The same arithmetic failure as an expression, which carries no frame. */
  when n = 24 then say n 'answered' s + 1
  /* Bare arithmetic, both halves of the pair. */
  when n = 25 then say n 'answered' s~'*'()
  when n = 26 then say n 'answered' s~'-'()
  when n = 27 then say n 'answered' s~'/'()
  when n = 28 then say n 'answered' s~'**'()
  when n = 29 then say n 'answered' '12'~'*'()
  when n = 30 then say n 'answered' '12'~'**'()
  when n = 31 then say n 'answered' '12'~'/'()
  when n = 32 then say n 'answered' '12'~'%'()
  when n = 33 then say n 'answered' '12'~'//'()
  when n = 34 then say n 'answered' '12'~'+'()
  when n = 35 then say n 'answered' '12'~'-'()
  when n = 36 then say n 'answered' ''~'*'()
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say s~'+'(1)
