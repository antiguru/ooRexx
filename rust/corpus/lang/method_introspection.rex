/* Method's own readers, over one directive of every kind that makes a method.
 *
 * WHAT SEPARATES A READER OF THE FLAGS FROM A READER OF NOTHING. The receiver
 * a first draft reaches for -- a primitive, .Object~method('objectName') --
 * answers an EMPTY ~source and six zeroes with isGuarded 1, so a body that
 * answered an empty array and six zeroes and a one would agree with the
 * oracle on nine rows and be wrong about every directive below it. Every
 * directive kind that makes a method is therefore here, and the flags
 * disagree with each other across them: abstract sets one bit, attribute
 * another, constant a third AND clears the guard bit no keyword touched, and
 * package and private are separate bits that setPrivate can leave both set.
 *
 * THE PATH IS NEVER PRINTED. ~package~name is this file's own absolute path;
 * corpus/lang/class_package.rex says why that may not reach stdout, and this
 * program projects it away the same way -- against PARSE SOURCE's third word,
 * which pins more than printing it would.
 *
 * THE FOURTH KIND OF RECEIVER IS THE ONE EASIEST TO MISS. ~define does not
 * store the object it is handed when that object already carries a scope: it
 * stores a COPY, a fresh handle in none of the tables the other three
 * receivers are in. Section J is that receiver, and it is the reason the copy
 * carries the record forward.
 *
 * ~source IS A LINE RANGE AND THE RANGE HAS TWO EDGES THE C++ PUTS THERE.
 * The block starts on the line after the directive and ends on the line
 * before the next one, so blank lines and whole-line comments INSIDE it are
 * its own -- ::method PAD below is the row that says so. And an end that
 * A `;` puts either edge of the range mid-line, and the four rows named for it
 * in section B are what say so: a clause the parser ended with `;` leaves the
 * block starting on its OWN line at the byte after it, and a directive that
 * does not begin its line cuts the line before it short. An implementation
 * answering whole lines only is right about every other row here.
 *
 * An end that
 * lands on an empty line steps back one (ProgramSource::extractSourceLines),
 * which is what the two blank lines below ::method STEP and the one at the end
 * of this file are for: STEP answers two of its three candidate lines and
 * TAIL, the last directive, answers one of its two. Delete that step and both
 * rows gain a line.
 */

parse source . . source

say 'A -- the seven flags, in the order Setup.cpp declares them'
call flags 'native', .Object~method('OBJECTNAME')
call flags 'written', .K~method('MM')
call flags 'unguarded-private', .K~method('UP')
call flags 'package', .K~method('PKG')
call flags 'protected', .K~method('PROT')
call flags 'guarded', .K~method('GUARD')
call flags 'abstract', .K~method('ABS')
call flags 'attribute-get', .K~method('ATT')
call flags 'attribute-set', .K~method('ATT=')
call flags 'constant', .K~method('CON')
call flags 'method-attribute', .K~method('MATT')
call flags 'attribute-with-body', .K~method('AB')
call flags 'external', .K~method('EX')

say 'B -- ~source, whose answer is an Array and not a string'
call lines 'native', .Object~method('OBJECTNAME')
call lines 'written', .K~method('MM')
call lines 'pad', .K~method('PAD')
call lines 'tail', .K~method('TAIL')
call lines 'abstract', .K~method('ABS')
call lines 'attribute-get', .K~method('ATT')
call lines 'constant', .K~method('CON')
call lines 'external', .K~method('EX')
call lines 'attribute-with-body', .K~method('AB')
call lines 'step', .K~method('STEP')
call lines 'semicolon-in-an-attribute-body', .K~method('SEMI')
call lines 'semicolon-on-the-directive-line', .K~method('SEMIDIR')
call lines 'a-directive-cutting-a-line-short', .K~method('CUT')
call lines 'after-that-cut', .K~method('AFTERCUT')

say 'C -- ~package, and the two packages a Method can belong to'
p = .Object~method('OBJECTNAME')~package
say 'native-package-class' p~class~id
say 'native-package-name' p~name
q = .K~method('MM')~package
say 'written-package-class' q~class~id
say 'written-package-name-is-the-source' (q~name == source)
say 'written-package-name-is-rexx' (q~name == 'REXX')
r = .K~method('PAD')~package
say 'second-method-package-name-is-the-source' (r~name == source)

say 'D -- the setters, each read back through its own is*'
m = .K~method('MM')
say 'start' m~isGuarded m~isPrivate m~isProtected
m~setUnguarded
say 'after-setUnguarded' m~isGuarded
m~setGuarded
say 'after-setGuarded' m~isGuarded
m~setProtected
say 'after-setProtected' m~isProtected
say 'source-survives-the-setters' m~source~items

say 'E -- setPrivate changes a send and not only a reader'
o = .K~new
say 'send-before' o~mm
m~setPrivate
say 'after-setPrivate' m~isPrivate m~isPackage
signal on syntax name blocked
say 'send-after' o~mm
say 'THE SEND WAS NOT BLOCKED'
exit 1

blocked:
say 'send-after-is-refused' rc
say 'F -- private and package are separate bits'
pk = .K~method('PKG')
say 'package-before' pk~isPackage pk~isPrivate
pk~setPrivate
say 'package-after' pk~isPackage pk~isPrivate

say 'G -- a Method no class has taken takes the setters too'
u = .methods~ZZ
say 'unattached-class' u~class~id
say 'unattached-guarded' u~isGuarded
u~setUnguarded
say 'unattached-after' u~isGuarded
say 'unattached-source' u~source~items

say 'H -- setSecurityManager answers the code object kind'
say 'native' .Object~method('OBJECTNAME')~setSecurityManager
say 'written' .K~method('MM')~setSecurityManager
say 'written-empty-list' .K~method('MM')~setSecurityManager()
say 'abstract' .K~method('ABS')~setSecurityManager
say 'attribute-get' .K~method('ATT')~setSecurityManager
say 'attribute-with-body' .K~method('AB')~setSecurityManager
say 'constant' .K~method('CON')~setSecurityManager
say 'external' .K~method('EX')~setSecurityManager
say 'still-answers-after' .K~method('MM')~isGuarded

say 'I -- a short argument list, and one argument too many'
call refuses 'flag-with-an-argument'
call refuses 'source-with-an-argument'
call refuses 'package-with-an-argument'

say 'J -- the fourth kind of receiver: the copy a ~define makes'
d = .K~method('UP')
.K2~define('X', d)
c = .K2~method('X')
say 'copy-class' c~class~id
say 'copy-scope' c~scope~id
say 'copy-flags' c~isAbstract c~isAttribute c~isConstant c~isGuarded c~isPackage c~isPrivate c~isProtected
cs = c~source
say 'copy-source-items' cs~items
do i = 1 to cs~items
  say '  <'cs[i]'>'
end
say 'copy-package-name-is-the-source' (c~package~name == source)

say 'K -- newFile, whose file is this one'
f = .Method~newFile(source)
say 'file-class' f~class~id
say 'file-scope' f~scope
say 'file-package-name-is-the-source' (f~package~name == source)
say 'file-has-source' (f~source~items > 0)
say 'file-flags' f~isAbstract f~isAttribute f~isConstant f~isGuarded f~isPackage f~isPrivate f~isProtected
say 'file-ssm' f~setSecurityManager
call refuses 'newFile-a-file-that-is-not-there'
call refuses 'newFile-with-no-argument'

say 'L -- loadExternalMethod, over the REXX library'
x = .Method~loadExternalMethod('file_separator', 'LIBRARY REXX')
say 'external-class' x~class~id
say 'external-scope' x~scope
say 'external-source' x~source~items
say 'external-package-name' x~package~name
say 'external-ssm' x~setSecurityManager
say 'external-flags' x~isAbstract x~isAttribute x~isConstant x~isGuarded x~isPackage x~isPrivate x~isProtected
say 'external-named-entry' .Method~loadExternalMethod('M9', 'LIBRARY REXX file_separator')~class~id
say 'external-lowercase-keyword' .Method~loadExternalMethod('M9', 'library REXX file_separator')~class~id
say 'external-mixed-keyword' .Method~loadExternalMethod('M9', 'LiBrArY REXX file_separator')~class~id
say 'external-tab-separated' .Method~loadExternalMethod('M9', 'LIBRARY	REXX	file_separator')~class~id
say 'external-unknown-entry' .Method~loadExternalMethod('M9', 'LIBRARY REXX')~string
call refuses 'loadExternal-a-descriptor-that-is-not-one'
call refuses 'loadExternal-a-descriptor-with-one-word'
call refuses 'loadExternal-a-descriptor-whose-first-word-is-not-LIBRARY'
call refuses 'loadExternal-a-descriptor-with-a-fourth-word'
call refuses 'loadExternal-an-empty-descriptor'
call refuses 'loadExternal-with-no-descriptor'
exit 0

flags:
  use arg tag, m
  say tag m~isAbstract m~isAttribute m~isConstant m~isGuarded m~isPackage m~isPrivate m~isProtected
  return

lines:
  use arg tag, m
  s = m~source
  say tag 'class' s~class~id 'items' s~items
  do i = 1 to s~items
    say '  <'s[i]'>'
  end
  return

refuses:
  use arg which
  signal on syntax name refused
  select
    when which == 'flag-with-an-argument' then say .K~method('MM')~isGuarded(1)
    when which == 'source-with-an-argument' then say .K~method('MM')~source(1)
    when which == 'package-with-an-argument' then say .K~method('MM')~package(1)
    when which == 'newFile-a-file-that-is-not-there' then say .Method~newFile('zzznosuchfile.rex')
    when which == 'newFile-with-no-argument' then say .Method~newFile()
    when which == 'loadExternal-a-descriptor-that-is-not-one' then say .Method~loadExternalMethod('M9', 'garbage')
    when which == 'loadExternal-a-descriptor-with-one-word' then say .Method~loadExternalMethod('M9', 'LIBRARY')
    when which == 'loadExternal-a-descriptor-whose-first-word-is-not-LIBRARY' then say .Method~loadExternalMethod('M9', 'REGISTERED junk')
    when which == 'loadExternal-a-descriptor-with-a-fourth-word' then say .Method~loadExternalMethod('M9', 'LIBRARY REXX file_separator extra')
    when which == 'loadExternal-an-empty-descriptor' then say .Method~loadExternalMethod('M9', '')
    otherwise say .Method~loadExternalMethod('M9')
  end
  say which 'WAS NOT REFUSED'
  return
refused:
  say which 'refused' rc
  return

::METHOD ZZ
  return 9

::CLASS K2 PUBLIC

::CLASS K PUBLIC

::METHOD MM
  return 1

::METHOD UP UNGUARDED PRIVATE
  return 2

::METHOD PKG PACKAGE
  return 3

::METHOD PROT PROTECTED
  return 4

::METHOD GUARD GUARDED
  return 5

::METHOD ABS ABSTRACT

::ATTRIBUTE ATT

::CONSTANT CON 6

::METHOD MATT ATTRIBUTE

::ATTRIBUTE AB GET
  a = 7
  return a

::METHOD EX EXTERNAL 'LIBRARY REXX file_separator'

::ATTRIBUTE SEMI GET
  a = 1;   b = 2
  return a + b

::METHOD SEMIDIR; return 11

::METHOD CUT
  return 12; ::METHOD AFTERCUT
  return 13

::METHOD PAD

  /* a comment line inside the block */
  return 8  /* and a trailing one on the clause */

  /* a comment line after the last clause */

::METHOD STEP
  return 10


::METHOD TAIL
  say 'tail-ran'; return 9   /* two clauses and a comment on one line */

