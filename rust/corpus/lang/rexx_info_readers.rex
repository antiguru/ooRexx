/* `.RexxInfo`'s readers, and the state each of them reads.
 *
 * The NUMERIC settings are changed FIRST and that is this program's point:
 * `.RexxInfo~digits`, `~form` and `~fuzz` are the interpreter's DEFAULTS and
 * not the settings in force, so a reader wired to the activation's live state
 * agrees with the oracle on every probe that leaves them alone. Measured,
 * oracle rc 0: `9 SCIENTIFIC 0` against `5 ENGINEERING 2` on the next line.
 * `.context`'s same-named methods are the live ones.
 *
 * The version fields are asserted as RELATIONS against `PARSE VERSION` rather
 * than printed. They come out of one constant, and printing them would commit
 * a differential over the oracle's build date that the next rebuild breaks --
 * which is also why no corpus program prints `parse version` itself. The
 * relations hold for whatever string a rebuilt oracle answers.
 *
 * The numeric-core limits are asserted against what the interpreter actually
 * enforces -- the exponent a literal stops converting past, the subscript
 * width that stops being read, the size `.Array~new` refuses -- so a body
 * answering a plausible number instead of the enforced one fails here even
 * though it would agree with a table that only compares the answer.
 *
 * `~package` is read twice: the object, then `~name`, because a reader that
 * answers a program's package instead of the REXX one renders differently but
 * a reader that answers some other native object may not -- and `~name` is
 * `REXX` for this package where a program's is that program's own path.
 *
 * The `select` at the end is the refusals, and the rows above it are the
 * adjacent successes that pin each bound: a subscript one digit wider than
 * `~internalDigits` and a size one past `~maxArraySize` are 93 where the row
 * at the bound answered, an argument to a method declared with a count of
 * zero is 93, and a name the class does not hold is 97.
 */

numeric digits 5
numeric form engineering
numeric fuzz 2
say 'defaults' .RexxInfo~digits .RexxInfo~form .RexxInfo~fuzz
say 'in force' digits() form() fuzz()

parse version v
d = .RexxInfo~date
say 'name is parse version' (.RexxInfo~name = v)
say 'date is its tail' (right(v, length(d)) = d)
say 'level is in it' (pos(' ' || .RexxInfo~languageLevel || ' ', v) > 0)
say 'version is in it' (pos(.RexxInfo~version, v) > 0)
say 'version is its parts' (.RexxInfo~version == .RexxInfo~majorVersion'.'.RexxInfo~release'.'.RexxInfo~modification)

say 'platform' .RexxInfo~platform .RexxInfo~architecture
say 'file system' .RexxInfo~directorySeparator .RexxInfo~pathSeparator .RexxInfo~caseSensitiveFiles .RexxInfo~maxPathLength
say 'end of line' c2x(.RexxInfo~endofline)
say 'debug' .RexxInfo~debug
say 'revision' .RexxInfo~revision

numeric digits 12
me = .RexxInfo~maxExponent
mi = .RexxInfo~minExponent
say 'exponents at the bound' datatype('1E+' || me, 'N') datatype('1E' || mi, 'N')
say 'exponents past it' datatype('1E+' || (me + 1), 'N') datatype('1E' || (mi - 1), 'N')

numeric digits 30
one = .Array~new(1)
id = .RexxInfo~internalDigits
say 'subscript at internal digits' one[copies('9', id)]
say 'internal digits' id
say 'internal max is its nines' (.RexxInfo~internalMaxNumber == copies('9', id))
say 'internal min is that negated' (.RexxInfo~internalMinNumber == ('-' || copies('9', id)))

p = .RexxInfo~package
say 'package' p
say 'package name' p~name

signal on syntax name refused
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' one[copies('9', id + 1)]
  when n = 2 then say n 'answered' .Array~new(.RexxInfo~maxArraySize + 1)
  when n = 3 then say n 'answered' .RexxInfo~digits(1)
  when n = 4 then say n 'answered' .RexxInfo~fileSeparator
  otherwise say 'done'
end
exit

refused:
say n 'refused' rc
signal on syntax name refused
signal next
