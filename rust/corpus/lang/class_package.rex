/* ~package, and the ~name of what it answers.
 *
 * The primitive classes belong to one package, whose name is REXX and which
 * renders as The REXX Package; a class a ::CLASS directive installed belongs
 * to its own file's package, which renders as a Package and whose name is
 * that file's path. So the pair of rows is what says ~package reads the
 * class rather than answering one object for everything.
 *
 * NO ROW HERE PRINTS THAT PATH. It is this program's own absolute path, which
 * would put the filesystem in the output and break the corpus's determinism
 * rule (README.md's "one rule", and the rule PARSE SOURCE's third word is
 * excluded by); corpus/lang/parse_sources.rex projects it away the same way.
 * Projected here by comparing it against PARSE SOURCE's third word instead,
 * which pins more than printing it would: the two strings are the same string,
 * and a build answering REXX for every class fails the row below that says so.
 */

say 'array-package' .Array~package
say 'array-package-name' .Array~package~name
say 'string-package-name' .String~package~name
say 'class-package-name' .Class~package~name
say 'k-package' .K~package
say 'k-package-class' .K~package~class
say 'k-package-isa-package' .K~package~isA(.Package)
parse source . . source
say 'k-package-name-is-the-source' (.K~package~name == source)
say 'k-package-name-is-rexx' (.K~package~name == 'REXX')
say 'array-package-name-is-the-source' (.Array~package~name == source)

::CLASS K
