/* A bare ::CLASS directive installs and is never entered. Phase 5a Task 4.
 *
 * ::CLASS naming nothing else -- no SUBCLASS, no METACLASS, no INHERIT --
 * neither runs code nor resolves a name against a table this crate does not
 * have, so it was already accepted before this task and stays accepted now
 * that ::CLASS/::METHOD/::ATTRIBUTE/::CONSTANT actually install. Measured,
 * rc 0, stdout "main ran".
 */

say 'main ran'

::class Foo
