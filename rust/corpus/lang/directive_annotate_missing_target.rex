/* An ::ANNOTATE whose target the accumulated package does not hold is
   99.945 at rc 157, echoing the ::ANNOTATE line, with stdout empty --
   Error_Translation_missing_annotation_target, raised while the directive
   list is read (parser/DirectiveParser.cpp:1983 and the four beside it).

   The target here is a class this file does declare, and declares BELOW the
   ::ANNOTATE. That is what makes the program a witness for the resolution
   order rather than for a typo: annotateDirective asks
   classDependencies->entry(name), a table addClassDirective fills as each
   ::CLASS is reached, so a build resolving against the whole file instead of
   against the part already walked runs this program at rc 0.

   say 'main ran' is the first clause and prints nothing, which is what says
   the refusal happened before the body rather than at the first send. */
say "main ran"

::annotate class K author "too early"

::class K
