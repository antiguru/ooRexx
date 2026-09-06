/* DATATYPE's refusals.
 *
 * One argument, and the only thing wrong with it is a first letter outside the
 * thirteen. That is 93.915 -- and DATATYPE is the one option argument in this
 * class whose message names the LETTER rather than the whole option string:
 * measured, 'Zonk' reports found "Z" here where strip('Zonk') reports found
 * "Zonk". An empty option has no first letter at all, and what the message
 * names is the NUL byte, rendered "?".
 *
 * The untrapped tail is the letter-only substitution. rc 163.
 */

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' '5'~dataType('Z')
  when n = 2 then say n 'answered' '5'~dataType('Zonk')
  when n = 3 then say n 'answered' '5'~dataType('')
  when n = 4 then say n 'answered' '5'~dataType(.nil)
  when n = 5 then say n 'answered' '5'~dataType('N', 1)
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
signal off syntax
say '5'~dataType('Zonk')
