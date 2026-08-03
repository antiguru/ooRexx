/* The error report echoes one clause per level, innermost first, and an
   INTERPRET fragment is a level.

   The shape at the bottom is chosen to make three different wrong answers
   print three different things, because a fragment raising at top level tells
   them apart from nothing:

   * The LINE. Both echoes carry the enclosing INTERPRET clause's line -- the
     last line of this file, whatever number that is. The fragment's own text
     is line 1 of a source of its own, so an implementation that resolves the
     clause's line where it resolves its TEXT prints 1 on the inner echo.

   * The INDENT of the outer echo. The INTERPRET sits inside one DO, so it
     echoes at 2.

   * The INDENT of the inner echo. The fragment nests its failing clause inside
     a DO of its own, so it echoes at 4 -- the enclosing clause's own 2 plus
     the fragment's own 2. An implementation that gave the fragment a level's
     worth of indent on top (the two spaces a CALLED ROUTINE really does get)
     prints 6; one that used the fragment's own depth alone prints 2.

   The inner echo's text is "say 2 & 1;" -- with the semicolon, because that
   is where the fragment's own clause span ends. Trimming it diverges.

   THE RULE ABOVE HAS ONE EXCEPTION and this file deliberately does not
   contain it: a DO that terminates by exhausting its iterations leaves the
   oracle's indent counter two spaces lower for every later clause at that
   level, which reaches straight through the fragment base -- interpret "do jj
   = 1 to 1; nop; end; say 1/0" one DO deep echoes at 0 on the oracle and 2
   here. That is a 4a divergence with nothing to do with fragments; see
   phase-4-exclusions.txt's KNOWN GAP row on the re-tested pass. The failing
   fragment below has no completed loop before its failing clause, so the
   plain rule applies to it. Anyone adding one to this file should expect it
   to diverge, and should not "fix" the indent to match.

   The successful INTERPRETs above it are there so the file is not only an
   error path: the nested one in particular runs a fragment from inside a
   fragment, which is the case that stacks three levels rather than two. It
   cannot be the failing one as well -- a raise ends the program -- so it is
   witnessed here for its output and the stacking depth is covered by
   rexx-exec's own unit tests. */

say 'start'
interpret "say 'one level'"
interpret 'interpret "say ''two levels''"'

do kk = 1 to 1
  interpret "say 'inside a DO'"
end

do kk = 1 to 1
  interpret "do jj = 1 to 1; say 2 & 1; end"
end
