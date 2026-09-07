/* A List index is converted with unsignedNumberValue at ARGUMENT_DIGITS, not
 * compared as an object: every spelling of one names entry 1, and a spelling
 * that will not convert is 93.918 rather than an answer.
 *
 * The handles of a three-item list are 0, 1 and 2, so 3 and above convert
 * and are absent -- .nil from `at`, 0 from `hasIndex` -- where a negative or
 * a fraction cannot convert at all and raises. That pair is what separates
 * "the index does not name an entry" from "the index is not an index", and
 * it is why `hasIndex('1e1')` answers 0 while `hasIndex('1e300')` raises.
 *
 * The last send is untrapped so the 93.918 text and the frame line are
 * compared as bytes. rc 163.
 */

l = .List~of('a','b','c')
say l~at(' 1') l~at('01') l~at(1.0) l~at('+1') l~at(1e0) l~at(' 1 ')
say l~hasIndex(' 1') l~hasIndex('01') l~hasIndex(1.0) l~hasIndex('-0')
say l~next('01') l~previous(' 1')
say 'converted and absent' l~hasIndex('1e1') l~hasIndex('1000000000')
say 'absent' l~at(99) l~hasIndex(99) l~next(99) l~previous(99) l~remove(99)

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' l~at('abc')
  when n = 2 then say n 'answered' l~at(-1)
  when n = 3 then say n 'answered' l~hasIndex(1.5)
  when n = 4 then say n 'answered' l~hasIndex('1e300')
  when n = 5 then say n 'answered' l~hasIndex('')
  when n = 6 then say n 'answered' l~remove('abc')
  when n = 7 then say n 'answered' l~next('abc')
  when n = 8 then say n 'answered' l~previous(.nil)
  when n = 9 then say n 'answered' l~index('abc')
  when n = 10 then say n 'answered' l~at(0)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say l~at('abc')
