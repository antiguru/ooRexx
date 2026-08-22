/* A nested array as a substitution item, on the ARRAY spelling. A list inside a
 * list is the only way to write one, so this shape became reachable when the
 * parenthesised list started evaluating.
 *
 * The item is rendered by stringValue() rather than by the string value a
 * string context asks for, which is where the two part: the report below reads
 * `in invocation of an Array` from slot one, and the traced element pair reads
 * `an Array` twice, where joining the inner list's elements reads `1` and `2`
 * on two lines. Slot two is empty and holds its place, so `maximum expected
 * is .`
 *
 * Untrapped, because the substituted message is the only channel that shows the
 * list, and traced and compared raw (see corpus.rs's RAW_STDERR_COMPARISON) for
 * the element lines. rc 216.
 */
trace i
raise syntax 40.4 array ((1,2),,'X')
