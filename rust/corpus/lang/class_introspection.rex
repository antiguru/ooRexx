/* Class's readers over its own graph, Object's three over a behaviour, and
 * the operators a class answers as messages.
 *
 * THE SUPPLIER'S ORDER IS HASH ORDER AND NOTHING HERE ASSERTS IT. The oracle
 * hands back a class's methods in neither alphabetical nor definition order,
 * and this crate hands them back sorted, because the two build the set from
 * different structures. String keys reproduce across runs of one interpreter
 * and nothing makes two interpreters agree, so a program that printed the
 * supplier in the order it arrives would diverge for a reason that is not a
 * defect. Every supplier row below therefore asserts MEMBERSHIP AND COUNT --
 * it walks the whole supplier, counts it, and picks out named entries by
 * comparing ~index -- and never prints one entry before another.
 *
 * WHY A COUNT IS SAFE HERE AND .Object~subclasses~items IS NOT. A class's
 * resolved method count is a property of that class: Object's is 32 on both
 * sides and a KK with two methods of its own is 34, and both move only if the
 * class set itself changes. The number of Object's subclasses is a property of
 * the whole image, so section E builds its OWN subclass and asks whether that
 * one is in the list rather than how long the list is.
 *
 * EVERY OBJECT-ANSWERING ROW IS READ THROUGH A SECOND SEND. ~methods answers a
 * Supplier, ~subclasses an Array, instanceMethod a Method and instanceMethods
 * a Supplier, and the arity instrument compares ~string, so a body answering
 * the right RENDERING with the wrong CLASS would agree with it. Each row here
 * reads ~class~id as well as a value, and the Method rows send ~scope~id to
 * the object they got back.
 *
 * THE OPERATORS HAVE TWO SPELLINGS AND BOTH ARE HERE. (.Array = .Array) is an
 * expression and .Array~'='(.Array) is a message; they take different paths
 * and a fix at one leaves the other refusing. Section A asserts both.
 */

say '-- A. the operators a class answers, expression spelling --'
say 'same     ' (.Array = .Array) (.Array == .Array)
say 'different' (.Array = .String) (.Array == .String)
say 'rendering' (.Array = 'The Array class') (.Array == 'The Array class')
say 'negated  ' (.Array \= .Array) (.Array \== .Array)
say 'notequal ' (.Array <> .String) (.Array >< .String)

say '-- A2. the same six, message spelling --'
say 'same     ' .Array~'='(.Array) .Array~'=='(.Array)
say 'rendering' .Array~'='('The Array class')
say 'negated  ' .Array~'\='(.Array) .Array~'\=='(.Array)
say 'notequal ' .Array~'<>'(.String) .Array~'><'(.String)

/* Concatenation never asked for an operator method and still does not, which
 * is the control for making a class an operator receiver: if the change had
 * widened past the comparison names these would have moved too. */
say '-- A3. concatenation, the control --'
say 'concat   ' (.Array || 'x')
say 'blank    ' (.Array 'x')

say '-- B. the readers, on a fresh subclass --'
k = .Object~subclass('K')
say 'isAbstract     ' k~isAbstract
say 'isMetaclass    ' k~isMetaclass
say 'queryMixinClass' k~queryMixinClass
say 'Class isMeta   ' .Class~isMetaclass

/* EVERY PREDICATE IS ASKED WHERE THE ANSWER IS 1 AS WELL AS WHERE IT IS 0.
 * The three rows above answer 0 and .Class~isMetaclass answers 1, so without
 * the two directives below a body returning a constant 0 for isAbstract and
 * queryMixinClass would satisfy every other row in this file. ABSTRACT and
 * MIXINCLASS are the only directives that set them. */
say '-- B2. the same predicates where the answer is 1 --'
say 'ABS isAbstract ' .ABS~isAbstract
say 'MIX queryMixin ' .MIX~queryMixinClass
/* Each predicate reads its own flag and not the other's, which one class
 * carrying one flag cannot show. */
say 'ABS queryMixin ' .ABS~queryMixinClass
say 'MIX isAbstract ' .MIX~isAbstract
say 'ABS isMetaclass' .ABS~isMetaclass

say '-- C. ~subclasses is an Array, read through a second send --'
sub = k~subclasses
say 'class          ' sub~class~id
say 'items          ' sub~items

say '-- D. ~methods is the WHOLE resolved set, not the class own --'
/* A body answering only the class's own dictionary gives 2 for KK where the
 * oracle gives 34; a body answering only Object's gives 32 for both. */
call countsupplier 'K  all      ', k~methods
call countsupplier 'KK all      ', .KK~methods
call countsupplier 'KK at KK    ', .KK~methods(.KK)
call countsupplier 'KK at Object', .KK~methods(.Object)
call countsupplier 'KK at nil   ', .KK~methods(.nil)
/* Not validated -- a scope that matches nothing is an empty answer, not a
 * raise. */
call countsupplier 'KK at 1     ', .KK~methods(1)

say '-- D2. the two entries KK owns, found by name and not by position --'
s = .KK~methods(.KK)
found1 = 0
found2 = 0
scopes = ''
do while s~available
  if s~index == 'M1' then found1 = 1
  if s~index == 'M2' then found2 = 1
  scopes = scopes s~item~class~id'/'s~item~scope~id
  s~next
end
say 'M1 present     ' found1
say 'M2 present     ' found2
say 'each item      ' scopes

say '-- E. a subclass registers in its parent list --'
/* Membership of a list this program itself extends, rather than the length of
 * one the whole image contributes to. */
base = .Object~subclass('BASE')
before = base~subclasses~items
kid = base~subclass('KID')
after = base~subclasses~items
say 'before         ' before
say 'after          ' after
list = base~subclasses
isthere = 0
do i = 1 to list~items
  if list[i]~id == 'KID' then isthere = 1
end
say 'KID in list    ' isthere
say 'list class     ' list~class~id

say '-- F. Object~isInstanceOf, over receivers of four kinds --'
o = .Object~new
say 'instance/Object' o~isInstanceOf(.Object)
say 'instance/String' o~isInstanceOf(.String)
say 'string/String  ' 'abc'~isInstanceOf(.String)
say 'integer/String ' 1~isInstanceOf(.String)
say 'class/Class    ' .Array~isInstanceOf(.Class)
say 'nil/Object     ' .nil~isInstanceOf(.Object)

say '-- G. instanceMethod answers a Method, asked a second question --'
im = o~instanceMethod('OBJECTNAME')
say 'class          ' im~class~id
say 'scope          ' im~scope~id
say 'scope class    ' im~scope~class~id
/* A name the receiver does not answer is .nil, not a raise. */
miss = o~instanceMethod('ZZZ')
say 'missing        ' miss~string
say 'missing is nil ' (miss == .nil)

say '-- H. instanceMethods, and the .nil scope that is NOT ~methods'"'"' --'
call countsupplier 'o all       ', o~instanceMethods
call countsupplier 'o at Object ', o~instanceMethods(.Object)
/* .nil selects the object's OWN level here and the receiving class in
 * Class~methods, which is the whole difference between the two entry points. */
call countsupplier 'o at nil    ', o~instanceMethods(.nil)
call countsupplier 'KK inst all ', .KK~new~instanceMethods
call countsupplier 'KK inst nil ', .KK~new~instanceMethods(.nil)

say '-- I. Class~enhanced renders as enhanced <id>, and the method works --'
tbl = .StringTable~new
tbl['EXTRA'] = .Method~new('EXTRA', 'return 99')
enh = k~enhanced(tbl)
say 'string         ' enh~string
say 'defaultName    ' enh~defaultName
say 'objectName     ' enh~objectName
say 'class id       ' enh~class~id
say 'the method runs' enh~extra
/* An EMPTY table installs no method and still renders enhanced, which is why
 * the flag is a flag and not the method list being non-empty. */
empty = k~enhanced(.StringTable~new)
say 'empty string   ' empty~string
say 'empty default  ' empty~defaultName
/* The neighbouring ordinary instance, which must NOT have moved. */
say 'plain instance ' k~new~string
say 'plain default  ' k~new~defaultName

say '-- J. the short argument list, beside the good one --'
call refuse "say .Array~isAbstract(1)"
call refuse "say .Array~isMetaclass(1)"
call refuse "say .Array~queryMixinClass(1)"
call refuse "say .Array~subclasses(1)"
call refuse "say .Array~methods(.Object, 2)"
call refuse "say .Object~new~isInstanceOf"
call refuse "say .Object~new~isInstanceOf(1)"
call refuse "say .Object~new~isInstanceOf(.Object, 2)"
call refuse "say .Object~new~instanceMethod"
call refuse "say .Object~new~instanceMethod('A', 'B')"
call refuse "say .Array~'='()"
/* Every operator a class does not answer, which is the oracle's own 97.1 and
 * not a refusal of this crate's. */
call refuse "say (.Array > .Array)"
call refuse "say (.Array + 1)"
call refuse "say (.Array ** 1)"
call refuse "say (.Array & 1)"
call refuse "say (-.Array)"
exit

/* Walks a supplier to its end and reports only its length -- never the order
 * the pairs arrive in. */
countsupplier:
  parse arg label
  sup = arg(2)
  n = 0
  do while sup~available
    n = n + 1
    sup~next
  end
  say label 'class' sup~class~id 'count' n
  return

/* Runs one clause and prints the condition it raised, so a refusal is a row
 * of output rather than the end of the program. */
refuse:
  parse arg clause
  signal on syntax name caught
  interpret clause
  say 'NO REFUSAL:' clause
  return
caught:
  say 'refused' condition('C') rc':' clause
  return

/* Two methods of its own, which is what makes ~methods' whole-set answer
 * distinguishable from an own-dictionary one: 34 against 2. */
::class KK
::method M1
  return 1
::method M2
  return 2

/* The two classes section B2 needs: the only directives that turn either
 * predicate on. */
::class ABS abstract
::class MIX mixinclass Object
