/* target~name:scope starts the lookup at scope and searches only the scopes
 * folded in ahead of scope, which is MethodDictionary::findSuperMethod.
 * Phase 5a Task 23.
 *
 * SUPER inside a method body is the scope RexxObject::superScope answers for
 * the scope the running method was found at, so self~tag:super in Mid's own
 * TAG reaches Base's. Every line here is class-side because ~new is 5b.
 */

say .Mid~tag                      /* the chain, one hop                  */
say .Deep~tag                     /* two hops, each through SUPER        */
say .Mid~tag:.Base                /* an ancestor named outright          */
say .Mid~tag:.Mid                 /* the receiver's own scope            */
say .Mid~tag:.Mixin               /* a scope folded in by INHERIT        */
say .Deep~tag:.Mid                /* an override that itself chains      */
say 'abc'~length:.String          /* a native method, its own scope      */
say .Mid~hasMethod:.Object("TAG") /* a scope merged from the metaclass   */
say (.Mid~~tag:.Base)~id          /* ~~ replaces the result by the target*/
say .Mid~nosuch:.Base             /* UNKNOWN is the ordinary unscoped one*/

::class Base
::method tag class
  return "base"
::method unknown class
  use arg name
  return "unknown:" name

::class Mixin mixinclass Object
::method tag class
  return "mixin"

::class Mid subclass Base inherit Mixin
::method tag class
  return self~tag:super"+mid"

::class Deep subclass Mid
::method tag class
  return self~tag:super"+deep"
