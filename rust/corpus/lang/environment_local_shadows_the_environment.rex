/* The order a .NAME resolves in, at the two steps this phase can observe:
 * .LOCAL before .ENVIRONMENT. Phase 5a Task 17.
 *
 * PackageClass::findClass asks the running package's own installed classes,
 * then .LOCAL, then .ENVIRONMENT, and only the last two are reachable from a
 * program that declares no ::CLASS. Setting an entry is itself a message send
 * to one of the interpreter's own objects, which is why this program could
 * not be written before the entry-method mechanism existed.
 *
 * The second name is the control: it is in .ENVIRONMENT alone, so a build
 * that stopped at .LOCAL falls through to the dotted text and a build that
 * asked .ENVIRONMENT first answers the wrong line above.
 *
 * Measured, rc 0.
 */

.local~MYTHING = 'from local'
.environment~MYTHING = 'from environment'
say .MYTHING

.environment~OTHERTHING = 'only in the environment'
say .OTHERTHING

/* And a name neither directory holds still renders as its own text. */
say .NEITHERTHING
