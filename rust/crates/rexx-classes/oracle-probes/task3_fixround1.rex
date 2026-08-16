/* Task 3, fix round 1. Every fact this program prints for Part A/B/D/E is
 * asserted verbatim (as the recorded value, not re-derived) by
 * crates/rexx-classes/tests/native_classes_wiring.rs -- except Set, Bag,
 * Relation and Supplier's own `OWN_INSTANCE` lines in Part A, which
 * `~inheritInstanceMethods` rescoping (see those four classes' own tests)
 * makes the wrong comparison for their pre-prologue derivation; that file's
 * tests instead assert their derived own set directly and check each
 * donated name's *absence* from the flattened set, not this printed line.
 * Part C's mixin-donation lines are asserted as presence/absence checks on
 * specific names, not as whole-line equality. Run as:
 *   ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
 *     /home/moritz/dev/repos/ooRexx/build/bin/rexx task3_fixround1.rex )
 * from a fresh scratch directory (never this one -- it sits on the external
 * routine search path from the repository root, so run a COPY elsewhere).
 * rc 0 expected; nothing here is in corpus/oracle-crashes.txt.
 */
numeric digits 18

/* Part A: every native class's own (scope-exact) instance method set, via
 * `cls~methods(cls)` sent to the class object itself -- no instantiation
 * needed, which is what resolves Pointer/Buffer (neither is constructible
 * via ~new with ordinary arguments; both answer this directly). */
call own_instance_methods 'Class'
call own_instance_methods 'Object'
call own_instance_methods 'String'
call own_instance_methods 'Array'
call own_instance_methods 'IdentityTable'
call own_instance_methods 'Table'
call own_instance_methods 'StringTable'
call own_instance_methods 'Set'
call own_instance_methods 'Directory'
call own_instance_methods 'Relation'
call own_instance_methods 'Bag'
call own_instance_methods 'List'
call own_instance_methods 'Message'
call own_instance_methods 'Method'
call own_instance_methods 'Routine'
call own_instance_methods 'Package'
call own_instance_methods 'RexxContext'
call own_instance_methods 'EventSemaphore'
call own_instance_methods 'MutexSemaphore'
call own_instance_methods 'MutableBuffer'
call own_instance_methods 'Supplier'
call own_instance_methods 'Pointer'
call own_instance_methods 'Buffer'
call own_instance_methods 'WeakReference'
call own_instance_methods 'StackFrame'

/* Part B: own class-method set, by hasmethod spot check (all of them are
 * small -- New alone, or New plus Of). */
call class_method_check 'Class', 'NEW'
call class_method_check 'Array', 'NEW'
call class_method_check 'Array', 'OF'
call class_method_check 'Set', 'NEW'
call class_method_check 'Set', 'OF'
call class_method_check 'Bag', 'NEW'
call class_method_check 'Bag', 'OF'
call class_method_check 'List', 'NEW'
call class_method_check 'List', 'OF'
call class_method_check 'Method', 'LOADEXTERNALMETHOD'
call class_method_check 'Method', 'NEWFILE'
call class_method_check 'Routine', 'LOADEXTERNALROUTINE'
call class_method_check 'Routine', 'NEWFILE'
call class_method_check 'Package', 'DEFAULTOPTIONS'

/* Part C: R8's measured subset -- for representative newly-native classes,
 * every derived (own) name is present on a live constructed instance (the
 * subset direction), and the mixin's own donated names are enumerated
 * exactly via a scope-exact query against the mixin class itself, now
 * `.NAME`-reachable because the prologue has already run in the oracle we
 * can probe. */
arr = .array~new(1)
call mixin_donation 'array_from_orderedcollection', arr, .orderedcollection

str = "foo"
call mixin_donation 'string_from_comparable', str, .comparable

st = .set~new
call mixin_donation 'set_from_mapcollection', st, .mapcollection
call mixin_donation 'set_from_setcollection', st, .setcollection
call mixin_donation 'set_from_setmixin', st, .setmixin

bg = .bag~new
call mixin_donation 'bag_from_manyitemmixin', bg, .manyitemmixin
call mixin_donation 'bag_from_bagmixin', bg, .bagmixin

rl = .relation~new
call mixin_donation 'relation_from_manyitemmixin', rl, .manyitemmixin

sup = .supplier~new(.array~new(1), .array~new(1))
call mixin_donation 'supplier_from_suppliermixin', sup, .suppliermixin

/* Part D: is_a, positive and negative. */
say 'isa_pointer_object=' .pointer~isa(.object)
say 'isa_pointer_method=' .pointer~isa(.method)
say 'isa_object_pointer=' .object~isa(.pointer)
say 'isa_array_object=' .array~isa(.object)
say 'isa_string_array=' .string~isa(.array)

/* Part E: the two method names that cannot survive Part A's space-joined
 * printout -- the concatenation operators. */
say 'object_hasmethod_empty=' .array~new(1)~hasmethod('')
say 'object_hasmethod_blank=' .array~new(1)~hasmethod(' ')
say 'string_hasmethod_empty=' "foo"~hasmethod('')
say 'string_hasmethod_blank=' "foo"~hasmethod(' ')

exit

::routine own_instance_methods
use arg n
cls = value('.'n)
sup = cls~methods(cls)
r = ''
do while sup~available
  r = r sup~index
  sup~next
end
say n'|OWN_INSTANCE=' wordsort(strip(r))
return

::routine class_method_check
use arg n, name
cls = value('.'n)
say n'|HASCLASSMETHOD('name')=' cls~hasmethod(name)
return

::routine mixin_donation
use arg label, inst, mixin_cls
sup = inst~instancemethods(mixin_cls)
r = ''
do while sup~available
  r = r sup~index
  sup~next
end
say label'=' wordsort(strip(r))
return

::routine wordsort
use arg s
a = s~makearray(' ')
a = a~sort
r = ''
do x over a
  if x <> '' then r = r x
end
return strip(r)
