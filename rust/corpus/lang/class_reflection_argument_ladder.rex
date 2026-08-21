/* The argument refusals of the reflection methods that take one, which name
 * their argument where the ones already in this crate number it: 88.909 for
 * ~method reports `argument method name` and not `Argument 1`, and 88.901 and
 * 88.914 report `class` for ~isA and ~isSubclassOf.
 *
 * Beside them the arity refusal, 93.902, which comes from the declared
 * parameter count and not from the method's body -- so it says `1 expected`
 * for both of them.
 *
 * The rows that answer are what stops "refuse every argument" from passing: a
 * class object is what ~isA and ~isSubclassOf want. Row 11 is the third
 * outcome and is why the number is there: a number has a string value, so it
 * reaches the lookup and raises 97.1 for its own digits rather than 88.909.
 *
 * The last send is untrapped so the 88.909 text and the frame line are
 * compared as bytes. rc 168.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' .Array~method()
  when n = 2 then say n 'answered' .Array~method(.nil)
  when n = 3 then say n 'answered' .Array~method(.Array)
  when n = 4 then say n 'answered' .Array~method('APPEND', 2)
  when n = 5 then say n 'answered' .Array~isA()
  when n = 6 then say n 'answered' .Array~isA('abc')
  when n = 7 then say n 'answered' .Array~isA(.Object, 2)
  when n = 8 then say n 'answered' .Array~isSubclassOf()
  when n = 9 then say n 'answered' .Array~isSubclassOf(5)
  when n = 10 then say n 'answered' .Array~isSubclassOf(.nil)
  when n = 11 then say n 'answered' .Class~method(5)
  when n = 12 then say n 'answered' .Array~isA(.Object)
  when n = 13 then say n 'answered' .Array~isSubclassOf(.Object)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say .Array~method(.nil)
