/* Method~scope answers the class a method object was defined at, and .nil for
 * one no class has taken (MethodClass::getScopeRexx, classes/MethodClass.cpp:361,
 * which is resultOrNil(getScope()), bound at Method alone by
 * memory/Setup.cpp:1113).
 *
 * gate-tables/concepts/xscope.rex is the pair that pins the answer to the
 * scope rather than to the ancestor a name is also defined at. What this file
 * adds is the other routes a program has to a method object, because each
 * reaches the scope field by a different write:
 *
 *   - an unattached ::METHOD, which .METHODS holds and no class has taken;
 *   - ~define with that object, which fills its scope in place;
 *   - ~define with an object that already carries a scope, which copies;
 *   - ~defineMethods, which copies for the reason its two newScope calls give
 *     (createMethodDictionary at classes/ClassClass.cpp:1265 and replaceMethods
 *     at MethodDictionary.cpp:233), leaving the object it was handed alone;
 *   - ::ATTRIBUTE's two accessors and ::CONSTANT, whose names no ::METHOD in
 *     this file spells;
 *   - a class Setup.cpp builds, and a name donated to one class out of
 *     another's dictionary, whose scope is the receiving class.
 *
 * The .nil row is what a build that fills the scope in unconditionally gets
 * wrong, and no other program here asks a method object that has no scope.
 *
 * The last send is untrapped and takes no trap, so the file pins the argument
 * refusal and the frame line a refusal raised inside this native method
 * carries -- which names the scope the method resolved at, and is the one
 * place the traceback prints it. rc 163.
 */

m = .methods~z
say 'unattached' m~scope
say 'unattached-class' m~scope~class~id

.k2~define('Y', m)
say 'define-fills-in-place' m~scope~id
say 'define-read-back' .k2~method('Y')~scope~id

.k3~define('X', .base~method('M'))
say 'define-copies' .k3~method('X')~scope~id
say 'donor-untouched' .base~method('M')~scope~id

.k4~defineMethods(.methods)
say 'define-methods' .k4~method('Z')~scope~id
say 'define-methods-copies' m~scope~id

say 'attribute-get' .base~method('A')~scope~id
say 'attribute-set' .base~method('A=')~scope~id
say 'constant' .base~method('C')~scope~id
say 'render' .base~method('M')~scope

/* APPEND is .Array's own, and AT is .StringTable's, donated into .Directory's
   own dictionary by InheritInstanceMethods(StringTable), memory/Setup.cpp:933. */
say 'library' .Array~method('APPEND')~scope~id
say 'donated' .Directory~method('AT')~scope~id

say .base~method('M')~scope(1)

::METHOD z
  return 'z'

::CLASS base
::METHOD m
  return 'base'
::ATTRIBUTE a
::CONSTANT c 5

::CLASS k2
::CLASS k3
::CLASS k4
