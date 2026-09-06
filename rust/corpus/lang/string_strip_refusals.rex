/* STRIP's refusals.
 *
 * The option is the only argument here with an error of its own: an
 * unrecognised letter is 93.915, and the message names the accepted set. The
 * character set has no error at all -- any length is legal, empty included --
 * so the only way to refuse it is to hand it a value with no string value,
 * which is 88.909 and belongs to the argument reader rather than to STRIP.
 *
 * The untrapped tail is the 93.915, whose substituted text names both the
 * accepted set and what was found -- and what it finds is the WHOLE option
 * string, not its first letter. That is where DATATYPE differs, and a
 * single-letter option cannot tell the two apart, so the tail spells a long
 * one. rc 163.
 */

signal on syntax name trapped
s = '  ab  '
n = 0

next:
n = n + 1
select
  /* An option outside "BLT". */
  when n = 1 then say n 'answered' s~strip('Z')
  when n = 2 then say n 'answered' s~strip('X')
  when n = 3 then say n 'answered' s~strip('')
  /* The message names the WHOLE option here, not its first letter -- which is
   * where DATATYPE differs, and a single-letter option cannot tell them
   * apart. */
  when n = 4 then say n 'answered' s~strip('Zonk')
  /* A value with no string value, in either position. */
  when n = 5 then say n 'answered' s~strip(.nil)
  when n = 6 then say n 'answered' s~strip('L', .nil)
  /* Two is the maximum. */
  when n = 7 then say n 'answered' s~strip('L', ' ', 1)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say s~strip('Zonk')
