/* .LOCAL, .ENVIRONMENT and .CONTEXT as objects, a class name out of the
 * environment, and the reflection names beside them. Phase 5a Task 6.
 *
 * .LOCAL and .ENVIRONMENT render as the names CoreClasses.orx:990 and :55
 * assign with ~objectName=, .ARRAY as RexxClass::defaultName's "The <id>
 * class", and .CONTEXT as RexxObject::defaultName's "a <id>".
 *
 * .METHODS, .ROUTINES and .RESOURCES are the string form of their own name
 * here rather than a StringTable, because the package declares no directive
 * of any of those kinds: LanguageParser hands the package its table only
 * when the table is non-empty, so the field stays null and the name falls
 * through. .RS is the same shape -- no command has set a return status.
 *
 * .LINE is the currently executing clause's own line number, so its answer
 * moves if a line is inserted above it.
 *
 * The last one resolves nowhere on either side and renders as its own
 * uppercased spelling with a period in front.
 *
 * Measured, rc 0.
 */

say .LOCAL
say .ENVIRONMENT
say .ARRAY
say .STRING
say .CONTEXT
say .METHODS
say .ROUTINES
say .RESOURCES
say .RS
say .LINE
say .noSuchEnvironmentName
