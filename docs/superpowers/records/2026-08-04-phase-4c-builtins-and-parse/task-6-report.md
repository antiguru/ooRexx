# Task 6 report: `builtin/numeric.rs`

`ABS FORMAT MAX MIN RANDOM SIGN TRUNC`, in a new `crates/rexx-exec/src/builtin/numeric.rs`.

Everything below was measured against
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx ABSOLUTE_PATH )`
from fresh empty scratchpad subdirectories, reading stdout, stderr and exit status as three
descriptors. `NUMERIC DIGITS` never exceeded 20.

---

## 1. The three flagged inferences

### (i) The `MAX`/`MIN` dispatch mechanism -- **read from the C++, then measured**

`BUILTIN(MAX)` (`interpreter/expression/BuiltinFunctions.cpp:1993`) is

```cpp
RexxObject *argument = get_arg(MAX, target);
if (isInteger(argument))       return ((RexxInteger *)argument)->Max(stack->arguments(argcount - 1), argcount - 1);
else if (isNumberString(...))  return ((NumberString *)argument)->Max(stack->arguments(argcount - 1), argcount - 1);
RexxString *target = required_string(MAX, target);
return target->Max(stack->arguments(argcount - 1), argcount - 1);
```

So the brief's inference is right: argument 1 is the **method target** and arguments 2+ are the
method's arguments. Both halves of the split follow directly.

* `RexxString::Max` (`classes/StringClass.cpp:1098`) goes through the `ArithmeticMethod` macro at
  `:1060`, whose failure is `reportException(Error_Incorrect_method_string_nonumber, name, this)`
  -- that is 93.943, and `name` is the literal `"MAX"`. That is why argument 1 answers with the
  builtin's own name where the rest do not.
* `NumberString::maxMin` (`classes/NumberStringMath.cpp:240`) loops the *method* arguments and
  raises `reportException(Error_Incorrect_method_number, arg + 1, args[arg])` with `arg` the
  0-based index into that list. The call's argument 2 is method argument 1. The "off by one" is
  the method/call numbering, exactly as the brief guessed.

**But the brief's account is incomplete, and the C++ says why.** There are *two* implementations,
not one, and they disagree about an omitted argument:

| path | omitted argument | position base | rc |
|---|---|---|---|
| `RexxInteger::Max` (`classes/IntegerClass.cpp:1578`), `requiredArgument(argument, arg)` | **93.903** | **0-based** | 163 |
| `NumberString::maxMin`, `reportException(Error_Incorrect_call_noarg, methodName, arg + 1)` | **40.5** | 1-based | 216 |

Measured, and the brief only saw the first:

```text
say max(1,,3)          93.903  Missing argument in method; argument 0 is required.        rc 163
say max(1,2,,4)        93.903  Missing argument in method; argument 1 is required.        rc 163
say max(1,2,3,,5)      93.903  Missing argument in method; argument 2 is required.        rc 163
say max(1.0,,3)        40.5    Missing argument in invocation of MAX; argument 1 ...      rc 216
say max('1',,3)        40.5    Missing argument in invocation of MAX; argument 1 ...      rc 216
say max(1e1,,3)        40.5    ...                                                        rc 216
say max(01,,3)         40.5    ...                                                        rc 216
say max(1,2.5,,4)      40.5    Missing argument in invocation of MAX; argument 2 ...      rc 216
```

`requiredArgument`'s own doc comment calls its second parameter "the position of the argument for
the error message", and the neighbouring `NumberString::maxMin` passes `arg + 1` for the identical
loop; `RexxInteger::Max` passes the raw loop index. So "argument 0" is two off-by-ones stacked,
not one.

**Which path a call takes is a property of the value's representation, and the rule is a
property of the literal's spelling.** `LanguageParser.cpp:2373` builds an integer object exactly
when `token->isIntegerConstant()`, which `Scanner.cpp:1546` sets for a run of digits no longer
than `Numerics::REXXINTEGER_DIGITS` (18) with no leading zero unless the whole symbol is `0`.
`RexxInteger::Max` then re-checks `Numerics::isValid(value, number_digits())`, so the *current*
precision can push a value back onto the general path:

```text
numeric digits 9; say max(12345,,3)     93.903 argument 0
numeric digits 3; say max(12345,,3)     40.5   argument 1
```

This is reproduced in `integer_object`/`valid_under`, keyed on the value's own rendering. See §6
for the two shapes this crate cannot get right and why.

### (ii) The three-level validation order -- **measured per builtin**

The order is `argument-2..n TYPE > argument-1 TARGET > argument-2..n RANGE`, and it holds for both
builtins that have optional numeric arguments. It is structural rather than chosen: the `BUILTIN`
body converts (40.12), `RexxString`'s `ArithmeticMethod` converts the target (93.943), and
`NumberString::trunc`/`formatRexx`'s own `optionalNonNegative` runs last (93.906).

```text
trunc('AB.CD','V')     40.12   TRUNC argument 2 must be a whole number; found "V".
trunc('AB.CD',-1)      93.943  TRUNC method target must be a number; found "AB.CD".
trunc(1.5,-1)          93.906  Method argument 1 must be zero or a positive whole number; found "-1".

format(1,'x')          40.12   FORMAT argument 2 must be a whole number; found "x".
format('a','x')        40.12   FORMAT argument 2 ...          -- the type check beats the target
format(1,'x','y')      40.12   FORMAT argument 2 ...          -- and runs front to back
format(1,1,'y')        40.12   FORMAT argument 3 ...
format('a',-1)         93.943  FORMAT method target must be a number; found "a".
format('a',1,-1)       93.943  ...                            -- the target beats every range
format(1,-1)           93.906  Method argument 1 ...
format(1,,-1)          93.906  Method argument 2 ...
format(1,,,-1)         93.906  Method argument 3 ...
format(1,,,,-1)        93.906  Method argument 4 ...
```

`ABS`, `SIGN`, `MAX` and `MIN` have no optional numeric argument, so only the target layer exists
for them. `RANDOM` is the exception in this family: it never reaches a method at all
(`context->random(...)` is on the activation), so **all** of its errors are 40.x at rc 216 and
there is no ordering between layers to establish -- its own order is seed-validity, then
reversed-range, then range-width (§4).

No value-before-length style inversion was found. The nearest thing is `RANDOM`'s, which is not an
inversion but a genuine three-way order, and it is pinned in
`random_validates_its_range_and_its_seed_separately`.

### (iii) The `2*DIGITS+1` negative-exponent trigger -- **read from the C++, refuting "a fit"**

It is not a fit; it is in the source twice.

* Default display: `NumberStringClass.cpp:391`,
  `if ((adjustedSize >= createdDigits) || (std::abs(numberExponent) > (createdDigits * 2)))`.
* `FORMAT`: `NumberStringClass.cpp:2029`, `adjustedLength >= exptrigger || (adjustedLength < 0
  && std::abs(numberExponent) > exptrigger * 2)`. The threshold is the same, with `exptrigger`
  in place of `createdDigits`, but the low-side arm carries an extra `adjustedLength < 0` guard
  that `:391` does not. That narrows *when* the arm is consulted and not the boundary it draws,
  so the conclusion is unaffected -- but "the same test" was the wrong description.

`abs(exponent) > 2*digits` for a negative exponent is `exponent <= -(2*digits + 1)`, which is
exactly what `Number::format` already implements (`rexx-num/src/lib.rs`, whose own comment records
the 78-of-1,674 differential result that found it). Note that the C++ compares the **raw**
exponent on the low side and the **adjusted** one on the high side -- the asymmetry `1e-18`
against `10e-19` shows -- and that it reads `createdDigits`, which is D15 stated in the
interpreter's own source. The trigger needed no change; the inference is confirmed, and its
status is upgraded from four data points to a citation.

---

## 2. The probe table

Every line was run through the oracle as a one-line program. `~` marks a newline inside the
captured stderr. The `Error NN running <path> line 1:` prefix is elided.

### `MAX` / `MIN` -- the family with zero suite coverage

| probe | stdout | rc | error |
|---|---|---|---|
| `max()` | | 216 | 40.3 `Not enough arguments in invocation of MAX; minimum expected is 1.` |
| `min()` | | 216 | 40.3 `... MIN; minimum expected is 1.` |
| `max(5)` | `5` | 0 | |
| `max(1,2,3,4,5,6,7,8)` | `8` | 0 | |
| `max('a',1,3)` | | 163 | 93.943 `MAX method target must be a number; found "a".` |
| `min('a',1)` | | 163 | 93.943 `MIN method target must be a number; found "a".` |
| `max('',1)` | | 163 | 93.943 `... found "".` |
| `max(' ')` | | 163 | 93.943 `... found " ".` |
| `max(1,'a',3)` | | 163 | 93.904 `Method argument 1 must be a number; found "a".` |
| `max(1,2,'a')` | | 163 | 93.904 `Method argument 2 ...` |
| `max(1,2,3,'a')` | | 163 | 93.904 `Method argument 3 ...` |
| `max(1,'')` | | 163 | 93.904 `Method argument 1 must be a number; found "".` |
| `max(1,,3)` | | 163 | 93.903 `Missing argument in method; argument 0 is required.` |
| `max(1,2,,4)` | | 163 | 93.903 `... argument 1 ...` |
| `max(1,2,3,,5)` | | 163 | 93.903 `... argument 2 ...` |
| `max(1.0,,3)` | | 216 | 40.5 `Missing argument in invocation of MAX; argument 1 is required.` |
| `max('1',,3)` | | 216 | 40.5 `... argument 1 ...` |
| `max(1.,,3)` / `max(1e1,,3)` / `max(01,,3)` | | 216 | 40.5 `... argument 1 ...` |
| `max(999999999999999999,,3)` (digits 9) | | 216 | 40.5 `... argument 1 ...` |
| `max(9999999999999999999,,3)` | | 216 | 40.5 `... argument 1 ...` |
| `max(word('1 2',1),,3)` | | 216 | 40.5 `... argument 1 ...` |
| `max(+1,,3)` / `max(-1,,3)` / `max(1 ,,3)` | | 163 | 93.903 `... argument 0 ...` |
| `x=1; max(x,,3)` | | 163 | 93.903 `... argument 0 ...` |
| `x='1'; max(x,,3)` | | 216 | 40.5 `... argument 1 ...` |
| `max(1+0,,3)` / `max(10/2,,3)` / `max(2*3,,3)` | | 163 | 93.903 `... argument 0 ...` |
| `max(1.0+0,,3)` | | 216 | 40.5 `... argument 1 ...` |
| `numeric digits 9; max(12345,,3)` | | 163 | 93.903 `... argument 0 ...` |
| `numeric digits 3; max(12345,,3)` | | 216 | 40.5 `... argument 1 ...` |
| `x=12345+0; numeric digits 3; max(x,,3)` | | 216 | 40.5 `... argument 1 ...` |
| `max(1,2.5,,4)` | | 216 | 40.5 `... argument 2 is required.` |
| `max(1,'a',,4)` | | 163 | 93.904 `Method argument 1 ...` (the omission is never reached) |
| `max(1.0,'a',,4)` | | 163 | 93.904 `Method argument 1 ...` |
| `max(1,2,'a',,5)` | | 163 | 93.904 `Method argument 2 ...` |
| `max(1.0,,'a')` | | 216 | 40.5 `... argument 1 ...` |
| `max(2,1e30)` | `1E+30` | 0 | |
| `max(1,2.5)` | `2.5` | 0 | |
| `min(2,1.5,0.5)` | `0.5` | 0 | |
| `max('0000000001',2)` / `max(2,'0000000001')` | `2` | 0 | |
| `max('1e2',5)` | `100` | 0 | |
| `max(' 1 ',5)` | `5` | 0 | |
| `max(1,1)` / `min(1,1)` / `max(-0,0)` / `max(0,-0)` | `1`/`1`/`0`/`0` | 0 | |
| `numeric digits 5; x=max(123456789,1); numeric digits 12; say x` | `1.2346E+8` | 0 | D15 |
| `numeric digits 5; x=min(123456789,1); numeric digits 12; say x` | `1` | 0 | D15 |
| `numeric form engineering; x=max(1e10,1); numeric form scientific; say x` | `10E+9` | 0 | D15/FORM |
| `max(1e10,1)` | `1E+10` | 0 | |
| `numeric digits 5; max(1.23456789,1)` | `1.2346` | 0 | |
| `numeric digits 3; max(12345)` | `1.23E+4` | 0 | |
| `numeric digits 3; min(12345)` | **`12345`** | 0 | the `MAX`/`MIN` asymmetry |
| `numeric digits 3; max(12345,6)` | `1.23E+4` | 0 | |
| `numeric digits 3; max(1.0e5,1)` | `1.0E+5` | 0 | |
| `numeric digits 9; numeric fuzz 3; max(100000000.0,100000001)` | `100000000` | 0 | FUZZ |
| `numeric digits 9; numeric fuzz 0; max(100000000.0,100000001)` | `100000001` | 0 | FUZZ |
| `numeric digits 9; numeric fuzz 3; min(100000000.0,100000001)` | `100000000` | 0 | |
| `numeric digits 9; numeric fuzz 3; max(1.000000000,1.000000001,0.999999999)` | `1.00000000` | 0 | |
| `numeric digits 3; max(1,12345+0)` | `1.23E+4` | 0 | the argument was already rounded by `+0` |
| `min(1,1.0)` | **`1`** | 0 | the tie that found this crate's one real bug |
| `max(1,1.0)` / `min(1.0,1)` / `max(1.0,1)` | `1`/`1.0`/`1.0` | 0 | |

### `ABS` / `SIGN`

| probe | stdout | rc | error |
|---|---|---|---|
| `abs(-4.5)` | `4.5` | 0 | |
| `abs('  -4.5  ')` | `4.5` | 0 | |
| `numeric digits 3; abs(1.23456)` | `1.23` | 0 | rounds even when already positive |
| `numeric digits 3; abs(-1.23456)` | `1.23` | 0 | |
| `numeric digits 3; x=abs(-1.23456); numeric digits 9; say x` | `1.23` | 0 | D15 |
| `numeric digits 3; x=abs(1.23456); numeric digits 9; say x` | `1.23` | 0 | D15 |
| `abs('abc')` | | 163 | 93.943 `ABS method target must be a number; found "abc".` |
| `abs('')` | | 163 | 93.943 `... found "".` |
| `abs(' ')` | | 163 | 93.943 `... found " ".` |
| `abs()` | | 216 | 40.3 `... minimum expected is 1.` |
| `abs(1,2)` | | 216 | 40.4 `... maximum expected is 1.` |
| `abs(17+'c')` | | **215** | 41.1 `Nonnumeric value ("c") used in arithmetic operation.` -- the `+`, not `ABS` |
| `sign(-12)` | `-1` | 0 | |
| `sign(0)` / `sign(-0.0)` | `0` | 0 | every spelling of zero is unsigned |
| `sign('abc')` / `sign(' ')` | | 163 | 93.943 `SIGN method target must be a number; ...` |
| `sign('-1E1234567890')` | | 163 | 93.943 `... found "-1E1234567890".` |
| `sign(-1E1234567890)` | | **215** | 41.1 -- the unary minus raises before `SIGN` is entered |
| `sign()` | | 216 | 40.3 |
| `numeric digits 3; x=sign(-1.23456); numeric digits 9; say x` | `-1` | 0 | D15 |
| `numeric digits 3; x=abs(-1.23456); numeric digits 9; say max(x,-1)` | `1.23` | 0 | the *stored* value is rounded |
| `numeric form engineering; abs(1e10)` | **`1E10`** | 0 | the literal's own text -- see D4 |
| `numeric form engineering; abs(-1e10)` | `10E+9` | 0 | |
| `numeric form engineering; abs('1e10')` | `10E+9` | 0 | |
| `numeric form engineering; abs(1.50)` | `1.50` | 0 | |
| `numeric form engineering; abs(0.000012345)` | `0.000012345` | 0 | |
| `numeric digits 3; abs(1e10)` | `1E+10` | 0 | |
| `numeric form engineering; sign(1e10)` | `1` | 0 | |
| `numeric form engineering; max(1e10)` | `10E+9` | 0 | |
| `numeric form engineering; trunc(1e10)` | `10000000000` | 0 | |

### `TRUNC`

| probe | stdout | rc | error |
|---|---|---|---|
| `trunc(12.987,2)` | `12.98` | 0 | truncates, does not round |
| `numeric digits 3; trunc(123456,2)` | `123000.00` | 0 | rounds to `DIGITS` **first**, no LOSTDIGITS |
| `numeric digits 9; trunc(1e20)` | `100000000000000000000` | 0 | never exponential |
| `trunc(1.5)` / `trunc(-1.5)` | `1` / `-1` | 0 | |
| `trunc(0,3)` | `0.000` | 0 | |
| `trunc(-0.0001234,2)` | `0.00` | 0 | the sign disappears with the digits |
| `trunc(1.23456789,'02')` | `1.23` | 0 | the spelling is not the value |
| `trunc(1,'1e3')` | `1` + 1000 zeros | 0 | |
| `trunc('AB.CD','V')` | | 216 | 40.12 |
| `trunc('AB.CD',-1)` | | 163 | 93.943 |
| `trunc(1.5,-1)` | | 163 | 93.906 |
| `trunc()` | | 216 | 40.3 `... minimum expected is 1.` |
| `trunc(1,2,3)` | | 216 | 40.4 `... maximum expected is 2.` |
| `numeric digits 3; x=trunc(1.23456,5); numeric digits 9; say x` | `1.23000` | 0 | text: nothing to move |
| `length(trunc(1,999999999))` | `1000000001` | 0 | |
| `trunc(1,4294967296)` | | **251** | Error 5 `System resources exhausted.` |
| `trunc(1,123456789012345678)` | | **251** | Error 5 |
| `length(trunc(1e999999))` | `1000000` | 0 | |
| `length(trunc(1e99999999))` | `100000000` | 0 | |
| `length(trunc(1e999999999))` | `1000000000` | 0 | see §6 for what this crate does |

### `FORMAT`

| probe | stdout | rc | error |
|---|---|---|---|
| `'['format(3.14159,2,3)']'` | `[ 3.142]` | 0 | |
| `'['format(-3.14159,5,2)']'` | `[   -3.14]` | 0 | |
| `format(12345.6789)` | `12345.6789` | 0 | |
| `format(12345.6789,,2)` | `12345.68` | 0 | |
| `format(1.234,6,2)` | `     1.23` | 0 | |
| `format(2.5,,0)` / `format(3.5,,0)` / `format(-2.5,,0)` | `3` / `4` / `-3` | 0 | half up away from zero |
| `format(1.245,,2)` | `1.25` | 0 | |
| `format(0)` | `0` | 0 | |
| `format(1,0)` | | 163 | 93.942 `Integer part of "1" is too large for 0 spaces.` |
| `format(0,0)` | | 163 | 93.942 `Integer part of "0" is too large for 0 spaces.` |
| `format(12345,,,0)` | `12345` | 0 | `expp=0` suppresses |
| `format(12345,,,,0)` | `1.2345E+4` | 0 | `expt=0` forces |
| `format(12345,,,0,0)` | `12345` | 0 | `expp=0` beats `expt=0` |
| `format(12345,,,2,0)` | `1.2345E+04` | 0 | |
| `format(12345,,,4,0)` | `1.2345E+0004` | 0 | |
| `format(1e10,,,,20)` | `10000000000` | 0 | |
| `numeric form scientific; format(1e10,,,,0)` | `1E+10` | 0 | |
| `numeric form engineering; format(1e10,,,,0)` | `10E+9` | 0 | |
| `numeric form engineering; format(0.000012345,,,,0)` | `12.345E-6` | 0 | |
| `numeric form engineering; format(123456,,,,0)` | `123.456E+3` | 0 | |
| `numeric form engineering; format(1234567,,,,0)` | `1.234567E+6` | 0 | |
| `format(1,,,,)` | `1` | 0 | every optional explicitly omitted is legal |
| `'['format(1,,,3)']'` | `[1]` | 0 | no exponent, so no field |
| `'['format(1,,,3,0)']'` | `[1     ]` | 0 | zero exponent reserved as blanks |
| `'['format(12345,,,3,0)']'` | `[1.2345E+004]` | 0 | |
| `format(.999999,,4,2,2)` | `1.0000    ` | 0 | the `bugs:#1474` path |
| `format(1e10,,,1,0)` | | 163 | 93.941 `Exponent of "1" is too large for 1 spaces.` |
| `format(1,'x')` | | 216 | 40.12 |
| `format('a',-1)` | | 163 | 93.943 |
| `format(1,-1)` / `(1,,-1)` / `(1,,,-1)` / `(1,,,,-1)` | | 163 | 93.906 method argument 1/2/3/4 |
| `format(1,,,,,1)` | | 216 | 40.4 `... maximum expected is 5.` |
| `format()` | | 216 | 40.3 `... minimum expected is 1.` |
| `format('abc')` | | 163 | 93.943 |
| `format('1e2')` / `format(' 1 ')` / `format('0000000001')` | `100` / `1` / `1` | 0 | |
| `numeric digits 3; format(123456,2)` | ` 1.23E+5` | 0 | rounded before `before` is applied |
| `format(1,3000000000)` | | **251** | Error 5 |
| `format(1,999999999)` | 999,999,999 bytes | 0 | |
| `format(1,,999999999)` | 1,000,000,001 bytes | 0 | |
| `format(1,,,999999999999999999)` | `1` | 0 | a width never materialised (§6) |
| `format(1,,,,999999999999999999)` | `1` | 0 | |
| `numeric digits 3; x=format(1.23456,,4); numeric digits 9; say x` | `1.2300` | 0 | text |

The 93.942 substitution needed its own probe set, because `rexx-num` had it wrong (§5):

| probe | 93.942 `&1` |
|---|---|
| `format(1.5,0)` | `1.5` |
| `format(1.5,0,0)` | `2` |
| `format(2.5,0,0)` | `3` |
| `format(0.4,0,0)` / `format(0.6,0,0)` | `0` / `1` |
| `format(9.5,0,0)` | `10` |
| `format(0.05,0,0)` / `format(0.05,0,1)` | `0` / `0.1` |
| `format(-1.5,1,0)` / `format(-0.5,1,0)` | `-2` / `-1` |
| `format(99.996,2,2)` / `format(99.996,2,0)` | `100.0` / `100` |
| `format(-99.996,2,2)` | `-100.0` |
| `format(12345.678,2,1)` | `12345.7` |
| `format(123456.789,2)` | `123456.789` |
| ENG `format(123456.789,2,,,0)` | `123.456789` |
| ENG `format(123456.789,2,3,,0)` | `123.457` |
| ENG `format(123456.789,2,0,,0)` | `123` |
| ENG `format(0.000012345,1,3,,0)` | `12.345` |
| `format(1e10,2,,0)` | `1E+10` |
| `numeric digits 3; format(1e10,2,,0)` | `1E+10` |
| ENG `format(1e10,0,,0)` | `10E+9` -- the substitution honours `FORM` too |

### `RANDOM`

| probe | stdout | rc | error |
|---|---|---|---|
| `random(5,5)` / `random(0,0)` | `5` / `0` | 0 | a degenerate range is legal |
| `random(,,)` | a number in 0..999 | 0 | trailing omissions are not arguments |
| `random(0,999999999)` | a number | 0 | the widest legal range; the check is `>`, not `>=` |
| `random(-1)` | | 216 | 40.33 `RANDOM argument 1 ("-1") must be less than or equal to argument 2 ("").` |
| `random(5,1)` | | 216 | 40.33 `... ("5") ... ("1").` |
| `random(999999999999999999)` | | 216 | 40.32 `RANDOM difference between argument 1 ("999999999999999999") and argument 2 ("") must not exceed 999,999,999.` |
| `random(0,1000000000)` | | 216 | 40.32 |
| `random(1,2,-1)` | | 216 | 40.13 `RANDOM argument 3 must be zero or positive; found "-1".` |
| `random(1,2,'-1.0')` | | 216 | 40.13 `... found "-1".` -- the *converted* value, not the text |
| `random('5.0','1.0')` | | 216 | 40.33 `... ("5") ... ("1").` -- likewise |
| `random(' -1 ')` | | 216 | 40.33 `... ("-1") ... ("").` |
| `random(1.5)` / `random('a')` | | 216 | 40.12 `RANDOM argument 1 must be a whole number; ...` |
| `random(1,2,3,4)` | | 216 | 40.4 `... maximum expected is 3.` |
| `random(1,999999999,12345)` | `776163098` | 0 | **reproducible across processes** |
| ...then two unseeded `random(1,999999999)` | `950445098`, `552120333` | 0 | the stream continues |
| `random()` as the program's first call | 894 / 152 / 414 on three runs | 0 | genuinely process-random |

---

## 3. Corpus axes, and how they were crossed

`crates/rexx-exec/src/builtin/numeric.rs`'s unit tests pin the individual behaviours; the coverage
claim rests on a differential sweep, run through `target/release/rexx-run` against
`build/bin/rexx` with both sides under the same `ulimit -v 1048576`, comparing stdout, stderr and
exit status as three separate byte streams. The generator is
`scratchpad/sweep.py`.

**Axes:**

1. **argument 1's alphabet, 45 values.** Signed and unsigned integers; values needing rounding at
   every `DIGITS` in the setting axis; both exponential ends (`1e10`, `1e-10`, `1E+100`,
   `-1E-100`); carry provokers (`0.999999`, `99.996`, `9.996E+20`, `1.999`); the shapes that
   decide the `RexxInteger` split (`01`, `1.0`, `1.`, `20.00`, `999999999999999999`); values whose
   spelling differs from their rendering (`'007'`, `'1e2'`, `'0000000001'`, `' 1 '`,
   `'  -4.5  '`); and the byte-string alphabet this phase requires of every probe set --
   **the null string `''`, a control byte `'00'x`, and two bytes >= 0x80 (`'ff'x`, `'80'x`)**,
   plus a tab-bearing value `'1'||'09'x` and a high-byte-bearing one `'1'||'ff'x`.
2. **the option grid.** `TRUNC`: 10 values for `places` including omitted, `0`, `-1`, a bad type
   `'x'` and a leading-zero spelling `'02'`. `FORMAT`: `before` x `after` x `expp` x `expt`, the
   full 6x5x5x4 = 600 combinations. `MAX`/`MIN`: 14 argument tails including interior omissions at
   three depths, a non-numeric among numerics, a mixed integer/non-integer list, an eight-argument
   list, and a `'00'x` argument.
3. **the settings.** `DIGITS` 1, 3, 5, 9 (default), 12, 20; `FORM` scientific and engineering;
   `FUZZ` 0 and 3.

**The crossing is a product, not a union.** Axis 1 x axis 2 x axis 3 runs in full for `ABS`,
`SIGN`, `TRUNC`, `MAX` and `MIN` -- 45 x 40 x 9. For `FORMAT` the 600-combination grid runs
against all 45 values under two of the nine settings (the default and `DIGITS 3`), and a
72-combination sub-grid -- still holding every position's *distinct kinds* of value: omitted, the
zero that means something special, a width, and the rejected negative -- runs against all 45
values under the other seven. So no axis is ever held at a safe value while another varies: every
setting sees every value with a grid containing each position's special cases, and every value
sees the full grid. Total 92,880 programs
(2 x 45 x 640 + 7 x 45 x 112), which the generator's own count agrees with.

`RANDOM` is deliberately absent from the sweep (D11: it must not appear in any corpus program).
Its determinism is pinned by unit test instead, against the oracle's own numbers.

**Result:**

```text
92880 programs, 18 mismatches
```

**All 18 are the two disclosed divergences of §6 and nothing else** -- 12 of D1 and 6 of D4. (The
D4 six are gone from the fix-round re-run, and §12 says why: the oracle itself was patched between
the two runs, not this crate.)

```text
numeric digits 1; max(-1.5,,3)   and 11 siblings ({max,min} x {-1.5,-2.5} x three omission depths)
numeric form engineering; abs(X) for X in 1e10, 1e-10, 1E+100, 9.996E+20, 01, 1.
```

The `abs` six are the same one thing: the oracle answers the *literal's own spelling* --
`[01]` where this crate answers `[1]`, and `[9.996E+20]` where this crate answers `[999.6E+18]`.

An earlier run of the identical 92,880 programs, against the build before the 93.942 substitution
was made `FORM`-aware (B6), reported **122** mismatches. The 104 that went away were all `FORMAT`
and all under `ENGINEERING`, and the fix that removed them is pinned by
`the_oversize_message_names_the_number_as_it_stands_at_the_failure`.

**Negative control.** The two defects this work actually had were reintroduced together --
`Extreme::op`'s `Min => CompareOp::Greater`, and `FormatError::BeforeOversize` substituting the
un-`after`-rounded value -- and the result built to a separate target directory so the good binary
was untouched. The **first 6,000 programs** of the same generator, which the fixed build matches
exactly (the progress line reads `... 5000 run, 0 mismatches` and the first mismatch is program
~28,800), report

```text
6000 programs, 155 mismatches
```

on the broken build. 155 against 0 over the identical inputs is what makes the zero mean
something. Both defects were originally found *by this sweep* rather than by review: `min(1,1.0)`
was the only mismatch in an early 1,200-program run, and the 93.942 substitution was 40 of the
first 4,000.

---

## 4. What was implemented, and the C++ it came from

* `ABS` -- `NumberString::abs` then `copyForCurrentSettings`: the sign is cleared and the value is
  rounded to the current `DIGITS`, *including* when it was already positive. Result is a number,
  so it captures the `DIGITS`/`FORM` pair (D15).
* `SIGN` -- `NumberString::Sign`: `-1`, `0` or `1`. No rounding is applied, because rounding cannot
  turn a non-zero value into a zero one; `Number::signum` answers `0` for every spelling of zero.
* `TRUNC` -- `NumberString::trunc`: `prepareNumber(digits, ROUND)` then `truncInternal`.
  `rexx-num`'s `Number::trunc` already implemented this and needed no change. Result is a
  `RexxString`.
* `FORMAT` -- `NumberString::formatRexx`/`formatInternal`. `rexx-num`'s `Number::format_with`
  already implemented the rendering; the `93.942` substitution needed fixing (§5). Result is a
  `RexxString`.
* `MAX`/`MIN` -- both paths, as §1(i) sets out. The general path compares through
  `rexx_num::compare_decoded` under `DIGITS` **and `FUZZ`**, which no other builtin in this family
  reads.
* `RANDOM` -- `RexxActivation::random`. The *stream* is reproduced exactly -- `RANDOM_FACTOR` 25214903917,
  `RANDOM_ADDER` 11, a supplied seed bit-inverted then scrambled 13 times, one further scramble per
  call, and the seed's bits reversed before the modulus. `DefaultRandomMin` 0, `DefaultRandomMax`
  999, `MaxRandomRange` 999999999. The *result's type* was not, and fix round 1 corrects it: the
  oracle answers a `RexxInteger` whose spelling is fixed, so the answer is text here (§11,
  Critical 2).
  * The seed is validated **and applied** before the range is looked at, which is the C++'s own
    first statement, so a call that then fails its range check has still advanced the stream.
  * `BUILTIN(RANDOM)`'s `argcount == 2 && both omitted` special case is **not** reproduced: it is
    unreachable (a trailing omission is not an argument, so `random(,)` arrives with none) and the
    values it substitutes are the defaults anyway. Stated in the code.
  * The state lives on `Interp`, one per interpreter, which is what
    `getRandomSeed`'s `isInternalLevelCall()` forwarding produces for the shapes 4c reaches.

New in `error.rs`: 40.13 (`argument_not_non_negative_call`), 40.32 (`random_range_too_wide`),
40.33 (`random_bounds_reversed`), 93.903 (`missing_method_argument`), 93.904
(`method_argument_not_a_number`), 93.943 (`method_target_not_a_number`), and
`impl From<FormatError> for Raised`. **93.906 already existed** (`argument_not_non_negative`, from
Task 3's `COPIES`/`INSERT`/`CHANGESTR`) and was reused; **41.1 already existed**
(`Raised::nonnumeric`) and was correctly *not* wired into `SIGN`, per the brief's item (b).

New in `rexx-num`: `Number::abs`, `Number::signum`, `FormatError::sub_code` (mirroring
`ArithError::sub_code`, which exists for the identical reason), and the `BeforeOversize` fix.

---

## 5. What the brief got wrong, and what `rexx-num` got wrong

**B1 (brief, incomplete).** Step 0(a) presents `max(1,,3)` -> 93.903 as *the* answer for an
omitted `MAX` argument. There are two answers, and which one a call gets depends on argument 1's
representation: `max(1.0,,3)` is **40.5 at rc 216**. Six of this task's probes distinguish them.
The brief's own framing ("the mechanism is unconfirmed -- read it") is what surfaced this.

**B2 (brief, missing).** `MAX` and `MIN` are not mirror images. `RexxInteger::Min` moves the
`argCount < 1` early return *above* the `Numerics::isValid` test and `RexxInteger::Max` does not,
with a source comment saying the move is deliberate so that `RexxInteger.testGroup` can tell the
two representations apart from inside Rexx. Measured: at `numeric digits 3`, `min(12345)` is
`12345` and `max(12345)` is `1.23E+4`.

**B3 (brief, understated).** Step 0(h) says the validation order is "measured for `TRUNC` and
`FORMAT` only". It is also the *only* order available to `ABS`/`SIGN`/`MAX`/`MIN`, which have no
optional numeric argument -- and `RANDOM` has no method layer at all, so its three errors are all
40.x. There is no fourth arrangement to find.

**B4 (`rexx-num`, a defect this task's sweep found).** `FormatError::BeforeOversize` substituted
the operand rounded to `DIGITS`; the interpreter substitutes bare `this`, which by then has been
reframed by the exponential decision **and** rounded by `after`. `format(1.5,0,0)` is
`Integer part of "2"` on the oracle and was `"1.5"` here. The doc comment asserting the opposite
cited `format(123456.789,2)` as its witness -- a call that never triggers exponential form, so
nothing was reframed for it to see. Fixed in `crates/rexx-num/src/format.rs`
(`reported_value`/`math_round_places`), with the false sentence corrected rather than hedged. The
cut needs `mathRound`'s digit-count-fixed carry, not `round_to_places`'s -- `format(99.996,2,2)`
reports `"100.0"` and not `"100.00"` -- which is the same distinction `post_carry_exponent_error`
already documented for 93.941.

**B6 (`rexx-num`, the same defect's second half, also found by the sweep).** `BeforeOversize`
rendered its substitution through `Number::format(digits)`, which is always SCIENTIFIC. The
interpreter renders through `stringValue()`, which honours `NUMERIC FORM`: measured,
`numeric form engineering ; format(1e10,0,,0)` reports `Integer part of "10E+9"` where the same
call under SCIENTIFIC reports `"1E+10"`. The variant now carries the `Form` as well as the
`digits`.

**B5 (this crate's own bug, found by the sweep and not by reading).** `MIN` needs
`CompareOp::Less`, not the negation of `CompareOp::Greater`: `Greater` is false for *equal* as
well as for *less*, so reading its `false` as "the candidate wins" swaps on every tie.
`min(1,1.0)` is `1` on the oracle and was `1.0` here. It was the only mismatch in the sweep's
first 1,200 programs.

---

## 6. Known divergences, disclosed

**D1. `MAX`/`MIN`'s two paths cannot be separated exactly, because this crate has no
`RexxInteger`.** `eval.rs` builds every source literal as text, so `1` and `'1'` are the same kind
of object here, and an arithmetic result is inlined by its *value* rather than by what its operands
were. The predicate used (`integer_object`, keyed on the value's own rendering, plus the
current-`DIGITS` check `valid_under`) agrees with the oracle on 18 of the 21 measured
discriminating probes. Two shapes fall the wrong way:

```text
max('1',,3)                          oracle 40.5 rc 216   here 93.903 rc 163
x='1'; max(x,,3)                     oracle 40.5 rc 216   here 93.903 rc 163
max(word('1 2',1),,3)                oracle 40.5 rc 216   here 93.903 rc 163
numeric digits 1; max(-1.5,,3)       oracle 40.5 rc 216   here 93.903 rc 163
```

The first three are a `RexxString` that reads as an integer; the fourth is a *computed*
non-integer whose rounded rendering is one (the unary minus makes `-1.5` a `NumberString`, which
`DIGITS 1` then rounds to `-2`). Keying on this crate's `SmallInt` tag instead -- its nearest
`RexxInteger` analogue -- agrees on only 14 of the 21, because a source literal is never one, so
the rendering-based rule is the better of the two available approximations. The whole class is
confined to an **interior omission** in a `MAX`/`MIN` call, a shape with zero coverage in the
whole ooTest suite.

**D4. `ABS` of a non-negative numeric *literal* under `NUMERIC FORM ENGINEERING` answers the
literal's own spelling, and this crate answers the canonical rendering.**

**These transcripts reproduce only against the oracle build that existed before 2026-08-05
16:02** -- see §12, which is the more important half of this row now. Measured:

```text
                             say abs(1e10)     1E+10
numeric form engineering ;   say abs(1e10)     1E10        <- the literal's own text, upcased
numeric form engineering ;   say abs(01)       01          <- and its leading zero with it
numeric form engineering ;   say abs(9.996E+20) 9.996E+20  <- where the canonical form is 999.6E+18
numeric form engineering ;   say abs(-1e10)    10E+9
numeric form engineering ;   say abs('1e10')   10E+9
numeric digits 3 ;           say abs(1e10)     1E+10
numeric form engineering ;   say max(1e10)     10E+9
numeric form engineering ;   say trunc(1e10)   10000000000
```

**This is an upstream defect, not a rule.** `NumberString::copyIfNecessary`
(`classes/NumberStringClass.cpp:3652`) decides whether to clone with
`digitsCount > digits || createdDigits != digits || isScientific() != form`, where `form` is
`number_form()` and `Numerics::FORM_SCIENTIFIC` is `false` while `FORM_ENGINEERING` is `true`
(`runtime/Numerics.cpp:108`). For a scientific-created number that reads `true != false` under
SCIENTIFIC -- always clone -- and `true != true` under ENGINEERING -- never clone. The comparison
is inverted; it wants `isScientific() != (form == Numerics::FORM_SCIENTIFIC)`. When it declines to
clone it returns the object itself, whose cached `stringObject` is the literal's own text, and
that is what prints.

Only `ABS` in this family reaches it: `MAX`, `MIN`, `TRUNC` and `FORMAT` all go through
`prepareNumber(digits, ROUND)`, which always clones, and `SIGN` discards the copy. Reproducing it
would need the same thing D1 needs -- a literal's parse-time-attached number told apart from a
runtime string's -- plus the inversion itself, so it is disclosed rather than imitated. **Not
filed upstream by this task.**

**D2. Retired in fix round 1.** It said `FORMAT`'s `expp` past `u32::MAX` was Error 5 here where
the oracle answers the short result. That was true, and it was also not the real boundary -- see
Critical 1 in §11. `expp` is now reserved exactly when the exponent field is written, which is the
line the oracle draws, and all of `format(1,,,3000000000)`, `format(1,,,999999999999999999)` and
`format(1,,,4294967296)` now answer `1` at rc 0 as the oracle does.

**D3. A result whose size comes from the *value's* magnitude rather than from an argument is not
guarded.** `trunc(1e999999999)` is a 1,000,000,000-byte string on the oracle at rc 0 under this
project's `ulimit -v 1048576`; here the same call goes through `rexx-num`'s infallible `String`
allocation. Under the memory asymmetry `rust/CLAUDE.md` documents -- **correction, 2026-08-08: this
had the subject backwards** -- this crate reserves 512 MiB for its interpreter stack, so the same
`ulimit` leaves this crate roughly half the room the oracle has -- the two sides cannot agree at
that scale anyway. `builtin::buffer` is applied to exactly what the brief asks for, the
argument-driven padding.

---

## 7. The three-state proofs

Each guard was mutated in place (restored from a copy, never `git checkout --`). "Without" is the
whole workspace suite with the named test `#[ignore]`d, which must stay **green** -- that is what
separates "adds coverage" from "can fail". "With" is the named test alone, which must go **red**
with a non-zero run count.

| guard mutated | test | without (want 0) | with (want != 0) | run count |
|---|---|---|---|---|
| `Extreme::op`: `Min => CompareOp::Greater` | `a_tie_keeps_the_earlier_value_at_both_ends` | 0 | 101 | 0 passed; 1 failed |
| `missing_method_argument(index)` -> `(index + 1)` | `an_omitted_argument_answers_differently_on_the_two_paths` | 0 | 101 | 0 passed; 1 failed |
| `integer_path`: drop `MIN`'s lone-target early return | `min_answers_a_lone_target_before_max_would` | 0 | 101 | 0 passed; 1 failed |
| `reported_value`: drop the `after` cut (`let cut = framed`) | `the_oversize_message_names_the_number_as_it_stands_at_the_failure` | 0 | 101 | 0 passed; 1 failed |
| `padding_width`: drop the `u32` check and the `buffer` reservation | `a_padding_width_too_large_to_allocate_raises_error_5` | 0 | 101 | 0 passed; 1 failed |
| `format`: move `target_number` above the four `whole_number` calls | `the_three_validation_layers_run_in_the_oracles_order` | 0 | 101 | 0 passed; 1 failed |
| `abs`: drop `round_to(digits)` | `abs_rounds_whatever_the_sign_and_sign_ignores_the_precision` | 0 | 101 | 0 passed; 1 failed |
| `next_seed`: return a fresh scramble instead of advancing the state | `a_seed_starts_a_stream_that_later_calls_continue` | 0 | 101 | 0 passed; 1 failed |
| `integer_path`: drop the `valid_under` check | `an_omitted_argument_answers_differently_on_the_two_paths` | 0 * | 101 | 0 passed; 1 failed |
| 93.904 -> 93.943 for arguments 2+ | `the_target_and_the_later_arguments_raise_different_numbers` | 0 * | 101 | 0 passed; 1 failed |

`*` -- these two are caught by **more than one** of this task's own tests, so ignoring only the
named one leaves the workspace red. Their "without" column was re-measured with
`cargo test --workspace -- --skip builtin::numeric`, which skips *every* test this task added: the
pre-existing tree is green under both mutations, so both tests genuinely add coverage rather than
duplicating something already there.

`abs`'s guard needed its test rewritten before it would go red at all. The first version asserted
only the rendered bytes, and a result built **without** `round_to` renders identically, because
`to_text` re-rounds through the captured `DIGITS` either way. The mutation survived it. What
distinguishes the two is a later read at a *wider* precision: measured, `numeric digits 3 ; x =
abs(-1.23456) ; numeric digits 9 ; say max(x,-1)` is `1.23`, so the rounding is in the stored
value. The test now feeds the result *object* back into `MAX` at `DIGITS 9` -- and passing its
bytes instead would re-round them and let the mutation through again.

An eleventh mutation was tried and is **not** listed, because it is not a guard: `padding_width`'s
`buffer` reservation and its `u32` bound are one guard, mutated together above.

---

## 8. Corpus rows moved

`rust/corpus/builtin-status.txt`: `ABS`, `FORMAT`, `MAX`, `MIN`, `RANDOM`, `SIGN` and `TRUNC` all
move `loud` -> `implemented`. None `divergent`. Re-derived by
`cargo test --offline -p rexx-exec --test builtin_status`, which asserts the file equals a live
differential run in both directions.

`rust/corpus/keyword-exempt.txt`: four rows **removed**, never re-attributed --
`CALL::test_args`, `IF::test_15`, `IF::test_16`, `IF::test_17`. The header count moves
`4c  770 bodies` -> `4c  766 bodies`. The failing total the harness reports moves 776 -> 772.

---

## 9. The verify block

Run from `rust/`, each exit status read unpiped.

```text
cargo test --offline --workspace --no-fail-fast          exit 0   1114 passed; 0 failed
cargo fmt --all --check                                  exit 0
cargo clippy --offline --workspace --all-targets -- -D warnings   exit 0
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus  exit 0
                                                         mode: STRICT (the gate)
                                                         42 of 42 matching
```

1,114 is 1,096 plus this task's 18 new tests. `cargo test --offline -p rexx-exec --test
builtin_status` is inside the workspace run and is what re-derives the seven flipped rows.

---

## 10. Commit

One commit, read back with `git log`:

```text
510a070e  Implement the seven numeric builtins
```

Working tree clean afterwards (`git status --short` empty). Files: `crates/rexx-exec/src/
builtin/numeric.rs` (new), `builtin/mod.rs`, `error.rs`, `lib.rs`, `crates/rexx-num/src/
format.rs`, `crates/rexx-num/src/lib.rs`, `crates/rexx-num/tests/format.rs`,
`corpus/builtin-status.txt`, `corpus/keyword-exempt.txt`. The sweep generator and the
mutation harness stayed in the scratchpad; nothing stray was staged.

---

## 11. Fix round 1

Two Criticals, two Importants and two Minors from
`task-6-review.md`. Every oracle run below used the same wrapper and a fresh empty scratchpad
subdirectory; `NUMERIC DIGITS` never exceeded 20.

### Critical 1 -- `FORMAT` panicked at `expp >= 65536` and aborted at `expp` near 3e9

**Two separate faults with one symptom.**

The panic was `rexx-num`'s, not the plumbing's: the exponent field was padded with
`format!("{:0width$}", …)`, and Rust's format width is a `u16`, so 65,536 is "Formatting argument
out of range" where the interpreter answers a 65,539-byte result. Replaced with an explicit
`"0".repeat(pad)`, which has no such ceiling.

The abort was the missed reservation. **`expp` is not like `before` and `after`**, and that is why
it was left out: the interpreter writes the exponent field only when there *is* an exponent to
pad, so reserving it eagerly turns a correct answer into Error 5. The oracle draws exactly that
line, and the two calls that differ only in `expt` show it:

```text
say format(1,,,3000000000)      1                              rc 0
say format(1,,,3000000000,0)    System resources exhausted.    rc 251
```

So the reservation is now made **conditionally**, on a question `rexx-num` answers rather than one
this crate guesses: a new `Number::format_exponent` returns the exponent `format_with` would
display, computed by the same two functions `format_with` itself runs (they share a new `trigger`
helper, so they cannot decide differently). When it says `None`, the width is dropped rather than
narrowed -- `rexx-num` reads `expp` only through the `== Some(0)` sentinel and the exponential
branch, and neither is reached. When it says `Some`, the width goes through `builtin::buffer` like
the other two.

**The threshold is the oracle's, not a plausible number**: the guard asks the allocator, which is
what the oracle does. Seventeen programs, run through both interpreters under the same
`ulimit -v 1048576`, three descriptors compared, **all matching**:

```text
length(format(1e10,,,65535))            65538                        rc 0
length(format(1e10,,,65536))            65539                        rc 0
length(format(1e10,,,100000))           100003                       rc 0
format(1,,,3000000000)                  1                            rc 0
format(1,,,4294967296)                  1                            rc 0
format(1,,,999999999999999999)          1                            rc 0
format(1,,,3000000000,0)                System resources exhausted.  rc 251
format(1,,,999999999999999999,0)        System resources exhausted.  rc 251
format(1e10,,,3000000000,0)             System resources exhausted.  rc 251
format(1e10,,,4294967296,0)             System resources exhausted.  rc 251
```

**This retires D2 entirely** rather than moving its boundary, which is what the review suspected
might happen. The in-code disclosure went with it.

The unit test uses a width of nine quintillion rather than three billion, and that is deliberate:
a `cargo test` process has no `ulimit -v`, so a 3 GB reservation genuinely succeeds there and then
builds a 3 GB string (it did -- the first version of this test produced an 11.2 GB failure
message). Only a width no allocator anywhere can supply asserts the same thing on every machine;
the 3e9 boundary is pinned differentially, above, where the limit is in force.

### Critical 2 -- `RANDOM` re-rendered its result under the current `DIGITS`/`FORM`

`RexxActivation::random` answers `new_integer(minimum)`, a `RexxInteger`, whose spelling is its
own decimal digits and is not re-rendered. Building it through `Interp::number` gave it the D15
pair instead, so a value wider than the precision came out exponential. Now built as text, the
rule `WORDS` and `LENGTH` already follow.

```text
numeric digits 1  ; say random(12345,12345)                12345
numeric digits 3  ; say random(12345,12345)                12345      (was 1.23E+4)
numeric digits 12 ; say random(12345,12345)                12345
numeric digits 3  ; numeric form engineering ; say ...     12345
numeric digits 3  ; say random(12345,12345) + 0            1.23E+4    the addition's result, not this one
```

All five match the oracle, plus `random(5,5)` and `random(0,0)`.

**The probe that now covers it, and the instruments extended.** The review's diagnosis was the
useful part: three checks were blind for three different reasons. Two of the three are now
extended.

* **`rust/corpus/builtin-probes.txt`** -- `RANDOM`'s row moves from `say random(5,5)` to
  **`numeric digits 3; say random(12345,12345)`**. This is the instrument that matters most,
  because it is committed, is re-derived against the oracle by
  `cargo test -p rexx-exec --test builtin_status`, and is re-run by every later task rather than
  only by this one. Its header gained a paragraph saying why a one-element range is not enough on
  its own: `random(5,5)` is 5 at every setting and cannot see a re-rendering, and this row really
  did read `random(5,5)` while the executor answered `1.23E+4`.
* **The unit tests** -- `a_random_answer_keeps_its_own_spelling_at_every_precision` runs the
  degenerate range at `DIGITS` 1, 3, 9 and 12 and under `ENGINEERING`, with
  `abs(-12345)` at `DIGITS 3` beside it as the adjacent contrast, so the test cannot pass on an
  implementation where nothing is ever reshaped.
* **The differential sweep** is *not* extended: D11 forbids `RANDOM` in a corpus program, and that
  rule is right -- an unseeded call is process-random on both sides. The sweep's blindness here is
  structural, and the answer is that the other two instruments have to carry it.

Two statements it made false are corrected rather than left: `numeric.rs`'s module doc now puts
`RANDOM` with `FORMAT` and `TRUNC` on the text side, and §4 above no longer says the generator is
"reproduced exactly" -- the *stream* is reproduced exactly (a seeded stream still matches the
oracle number for number), the result's *type* was not.

### Important 3 -- the divergences had no row in the plan's live record

Three rows added to `docs/superpowers/plans/phase-4-exclusions.txt`, in `KNOWN GAPS`, each with
its measurement and its owner:

* **`MAX`/`MIN` -- which of two implementations runs depends on a type this crate does not have.**
  Carries both C++ receivers with citations, the four-line transcript of the split, the parse-time
  rule that selects one, the four shapes this crate gets wrong, why the `SmallInt` alternative is
  worse (14 of 21 against 18 of 21), and the sweep count. No owner assigned.
* **`ABS` of a positive literal under `ENGINEERING`.** Stated plainly as an **upstream ooRexx
  defect**, with the `copyIfNecessary` citation and the `FORM_SCIENTIFIC == false` inversion
  spelled out. **Recorded, not filed** -- that decision is Moritz's. No owner assigned.
* **A result sized from a value's own magnitude is not reserved fallibly** (D3). Says what closing
  it needs and why no threshold is quotable under `ulimit -v`. No owner assigned.

D2 needed no row: it no longer exists.

### Important 4 -- D2's in-code disclosure

Gone with the divergence. `format`'s comment now says what the code does -- `before` and `after`
are always written and always reserved; `expt` is a threshold and occupies no bytes; `expp` is
reserved only once `format_exponent` says the field will be written -- with the oracle transcript
that draws the line.

### Minor 5 -- `NumberStringClass.cpp:2029`

Corrected in §1(iii). It is the same threshold with `exptrigger` in place of `createdDigits`, but
the low-side arm carries an extra `adjustedLength < 0` guard that `:391` does not. The conclusion
is unaffected; the description was wrong.

### Minor 6 -- `reported_value` on the dead path

Taken. `format_with` now computes it only when `before` is supplied, which is the only way the
raise that reads it can be reached, and `render_integer_padded` takes `Option<&Number>` so the
pairing is in the type rather than in a comment.

### Fix-round verification

```text
cargo test --offline --workspace --no-fail-fast          exit 0   1115 passed; 0 failed
cargo fmt --all --check                                  exit 0
cargo clippy --offline --workspace --all-targets -- -D warnings   exit 0   (clean target dir)
REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus  exit 0
                                                         mode: STRICT (the gate)
                                                         42 of 42 matching
```

1,115 is round 0's 1,114 plus the new `RANDOM` rendering test. Clippy was run with
`CARGO_TARGET_DIR` pointed at an empty directory, per `rust/CLAUDE.md`'s rule that a warm green is
provisional.

**Commit:** `2472fd8a` -- "Stop FORMAT aborting on a wide expp, and fix RANDOM's rendering",
read back with `git log`; working tree clean afterwards. Files: `crates/rexx-exec/src/builtin/
numeric.rs`, `crates/rexx-num/src/format.rs`, `corpus/builtin-probes.txt`,
`docs/superpowers/plans/phase-4-exclusions.txt`.

Nothing in this round changed a success path the round-0 sweep covered except `FORMAT`'s `expp`
handling, and the ten-program differential above is that change's own witness. The `MAX`/`MIN`
comparison, the 93.942 substitution, `TRUNC`, `ABS` and `SIGN` are untouched.

The full 92,880-program sweep was re-run against the fixed build all the same:

```text
92880 programs, 12 mismatches
```

all twelve D1, all of the shape `numeric digits 1; {max,min}(-1.5|-2.5, ...)` with an interior
omission. **The six D4 mismatches are gone, and not because of anything in this round** -- §12.

---

## 12. The oracle was patched and rebuilt mid-task

**Found while confirming fix round 1, and it is the most important thing in this report.**

The fix-round confirmation sweep -- the same 92,880 programs, the same generator -- reported **12**
mismatches where round 0's reported **18**. Nothing in fix round 1 touched `ABS`, so the six that
vanished could not have been my doing. They were not:

```text
$ ls -la --time-style=full-iso /home/moritz/dev/repos/ooRexx/build/bin/rexx
-rwxrwxr-x 1 moritz docker 62600 2026-08-05 16:02:20 .../build/bin/rexx
$ ls .../build/lib/rexx.img          2026-08-05 16:02
$ cd /home/moritz/dev/repos/ooRexx && git status --short
 M interpreter/classes/NumberStringClass.cpp
```

The modification is uncommitted, and it is exactly D4:

```diff
-    if (digitsCount > digits || createdDigits != digits || isScientific() != form)
+    // NOTE: the form flag and the form constants have opposite polarity
+    // (FORM_SCIENTIFIC is false), so these must be compared via the constants,
+    // not by testing isScientific() against the form directly.
+    if (stringObject != OREF_NULL || digitsCount > digits || createdDigits != digits ||
+        (form == Numerics::FORM_SCIENTIFIC && isEngineering()) ||
+        (form == Numerics::FORM_ENGINEERING && isScientific()))
```

**Task 6 did not make this change**; it treats that tree as read-only and has not written to it.
The same program, byte for byte, that answered `1E10` earlier in this session now answers `10E+9`
six runs out of six, from the original file and from a fresh copy in a clean directory.

**What it does and does not invalidate.**

* `copyIfNecessary` is reached from `NumberString::abs` and `NumberString::Sign` and from nothing
  else this family touches. `Sign` discards the copy and keeps only the sign, so **`ABS` is the
  only builtin here whose behaviour the patch can change.** `MAX`, `MIN`, `TRUNC` and `FORMAT` all
  go through `prepareNumber(digits, ROUND)`, which always clones.
* Every result in §§1-11 was therefore re-checked against the **current** binary, not carried
  over: the seventeen-program Critical 1 and 2 differential (all matching), `builtin_status`
  (12 passed, which re-derives all 66 rows live), and the corpus gate (42 of 42). The full
  92,880-program sweep in §11 already ran entirely after the rebuild.
* D4's own transcripts are the one thing that no longer reproduces. The finding was right --
  someone read it and fixed it -- and this crate answered the canonical `10E+9` throughout, which
  is what the patched oracle now answers too. So the six mismatches are gone because the
  *reference* moved, not the implementation.
* The `KNOWN GAP` row for D4 now carries all of this, and stays: the patch is uncommitted in a
  tree this project does not own, so reverting it brings the divergence back, and a reader
  comparing against a pristine ooRexx checkout will see the original transcripts.

**The general point, which outlives this patch.** A rebuilt oracle silently reprices every
differential result recorded against the old one, and **nothing in the harnesses notices** --
`corpus.rs`, `builtin_status.rs` and the sweep all re-run the binary they find and compare against
it, so a changed reference reads as a changed implementation. The two are told apart only by
someone remembering to look at a timestamp. A build fingerprint recorded beside a differential
result would close that, and no instrument here has one.

**Reported, not acted on.** Whether the patch should be committed, upstreamed, or reverted is
Moritz's call, and this task has not touched the tree either way.
