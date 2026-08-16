/* .METHODS is the string form of its own name when every method in the file
 * is attached to a class. Phase 5a Task 6.
 *
 * The negative half of the witness beside this one. A ::METHOD under a
 * ::CLASS goes into that class's dictionary and never into the package's
 * unattached table, so the table stays empty, LanguageParser leaves the
 * package's field null, and the name falls through to its own text -- a
 * build that answered a StringTable whenever a ::METHOD existed anywhere
 * would print the wrong line here and the right one there.
 *
 * Measured, rc 0.
 */

say .METHODS

::class Holder
::method attachedMethod
  return 1
