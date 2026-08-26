/* The argument refusals of Class~subclass and Class~mixinClass, and the
 * successful shapes they are read against.
 *
 * The order inside RexxClass::subclass is what decides which refusal a send
 * owing more than one gets, and rows 1 and 5 are that boundary: the metaclass
 * is tested first (ClassClass.cpp:1572), so a send with no class id and a bad
 * metaclass reports the metaclass. Row 4 is why the test is not "omitted or
 * .nil" -- the C++ tests meta_class == OREF_NULL, which an omitted argument
 * is and a supplied .nil is not.
 *
 * 93.902 comes from the declared parameter count and not from the body, and
 * both spellings declare the same count, which is what makes rows 6 and 9 the
 * same number. The ladder compares numbers; the text of one of them is
 * compared by the untrapped row at the end.
 *
 * Row 12 is ~inherit's own refusal reached through a class this file built,
 * which is what says the mixin flag and the base class took: a MIXINCLASS
 * built on .Array may only be inherited by a subclass of .Array.
 *
 * The last send is untrapped so the 88.901 text and the frame lines are
 * compared as bytes. The frame pair is the point: the class id is checked
 * inside RexxClass::newRexx, which subclass reaches by sending NEW to the
 * metaclass (ClassClass.cpp:1576), so the report carries NEW's frame under
 * SUBCLASS's own. rc 168.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' .object~subclass()~id
  when n = 2 then say n 'answered' .object~subclass(.environment)~id
  when n = 3 then say n 'answered' .object~subclass("Kay", .Object)~id
  when n = 4 then say n 'answered' .object~subclass("Kay", .nil)~id
  when n = 5 then say n 'answered' .object~subclass(, "abc")~id
  when n = 6 then say n 'answered' .object~subclass("Kay", .Class, .methods, 'e')~id
  when n = 7 then say n 'answered' .object~mixinclass()~id
  when n = 8 then say n 'answered' .object~mixinclass("Kay", .Object)~id
  when n = 9 then say n 'answered' .object~mixinclass("Kay", .Class, .methods, 'e')~id
  when n = 10 then say n 'answered' .object~subclass(7)~id
  when n = 11 then say n 'answered' .object~subclass("Ok1")~id .object~mixinclass("Ok2")~baseClass~id
  when n = 12 then say n 'answered' .object~subclass("Nay")~inherit(.array~mixinclass("Mix"))
  when n = 13 then say n 'answered' .object~subclass("Ok3", .Class)~id
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say .object~subclass()~id

::METHOD z
  return 'z'
