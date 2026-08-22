/* provide.xml `reqstr`'s "for all other methods" rule: when a method expects a
 * string, the argument is sent request("STRING") and the method raises if that
 * answers .nil. So a makeString on the argument's class puts an object on the
 * answering side of the 88.909 line that
 * message_send_argument_object_not_a_string.rex draws, and this program is that
 * line re-derived from the section rather than from a coercion audit: the audit
 * asked which value *shapes* have a string value, where the section says the
 * question is a message send.
 *
 * Every argument position a primitive method in this phase declares as text is
 * here, because each raises its own message: a numbered argument, an argument
 * the message names, and an option letter.
 *
 * The other half of the section's method rule -- String's arithmetic,
 * comparison and concatenation methods falling back to ~string rather than
 * raising -- has no row anywhere: this crate implements none of those methods,
 * so `'abc'~pos(.k)` is a refusal rather than an answer.
 *
 * Phase 5a Task 14.
 */

say 'hasMethod' 'abc'~hasMethod(.length)
/* A message instruction, because a Method object answers no message this
   phase implements and there is nothing printable to do with the result. */
.k~method(.m)
say 'method answered'
say 'directory at' .environment~at(.arrayname)
.environment~put('stored', .m)
say 'directory read back' .environment~at('M')
say 'array option' .Object~superClasses~makeString(.option)
.p~objectName = .m
say 'objectName=' .p~objectName

n = 0
signal on syntax name trapped

next:
n = n + 1
select
  when n = 1 then say 'abc'~hasMethod(.p)
  when n = 2 then say .environment~at(.p)
  when n = 3 then say .k~method(.p)
  when n = 4 then say .Object~superClasses~makeString(.p)
  when n = 5 then say 'abc'~hasMethod(.notanoption)
  when n = 6 then say .Object~superClasses~makeString(.notanoption)
  otherwise signal done
end
say n 'answered'
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say 'done'
exit 0

::class p

::class length
::method makeString class
  return 'LENGTH'

::class m
::method makeString class
  return 'M'

::class arrayname
::method makeString class
  return 'ARRAY'

::class option
::method makeString class
  return 'C'

::class notanoption
::method makeString class
  return 'zz'

::class k
::method m
  return 'k m'
