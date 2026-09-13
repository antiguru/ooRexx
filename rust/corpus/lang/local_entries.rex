/* Every `.local` name the oracle holds, read as an entry rather than as a
   dot-variable.

   The two spellings must answer the same object: `.output` mints the bundle
   on demand and `.local['OUTPUT']` is the same demand, so a reader that only
   minted for one of them would answer a monitor for the first and nothing for
   the second. `STDQUE` is left out -- the external queue is not built, and its
   entry refuses rather than answering. */

do name over .array~of('DEBUGINPUT', 'ERROR', 'INPUT', 'OUTPUT', 'STDERR', 'STDIN', 'STDOUT', 'SYSCARGS', 'TRACEOUTPUT')
  say name .local[name]~class~id
end

-- `.ENDOFLINE` is `.environment`'s, not `.local`'s, and is here because the
-- ooTest framework's own prologue reads it: the suite stopped on it.
say 'endofline' c2x(.endOfLine) length(.endOfLine)
say 'input  ' (.local['INPUT'] == .input)
say 'stdout ' (.local['STDOUT'] == .stdout)
say 'wrapped' (.debuginput~current == .input)
