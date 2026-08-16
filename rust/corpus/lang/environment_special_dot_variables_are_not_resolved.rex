/* .NIL, .TRUE and .FALSE are resolved by the parser and never reach the
 * environment, and VALUE's one-argument form does reach it. Phase 5a Task 6.
 *
 * LanguageParser's constructor puts a SpecialDotVariable retriever in the
 * dot-variable table for each of those names, so the expression form is a
 * constant and a class of that name cannot shadow it. VALUE goes through
 * getVariableRetriever instead and gets an ordinary dot variable, which
 * resolves -- and finds this file's own class.
 *
 * The pair on each name is the point: the same spelling answers differently
 * on the two routes, and a build that shared one resolution for both would
 * get one of the two lines wrong.
 *
 * Measured, rc 0.
 */

say .TRUE
say value('.TRUE')
say .NIL
say value('.NIL')

::class True
::class Nil
