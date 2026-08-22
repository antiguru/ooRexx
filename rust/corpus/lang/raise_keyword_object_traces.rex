/* The condition keyword's own value line when the error code is an object.
 * `traceKeywordResult(conditionName, rc)` is handed the object and
 * `rc->requestString()` is separately what becomes the code, so the line reads
 * `an Array` while the code is the items joined -- and the joined text is not a
 * valid error code, which is the rc 223 report below.
 *
 * Untrapped and traced, in corpus.rs's RAW_STDERR_COMPARISON, because the
 * `>K>` line is the whole subject.
 */
trace i
raise syntax (1,2)
