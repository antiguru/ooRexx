/* QUALIFY against the interpreter's own current directory, and one SETLOCAL
   pair. Exactly one ENDLOCAL restores: the oracle aborts on the second restore
   in a process (corpus/oracle-crashes.txt entry 10), so a second pair here
   would be a program no differential may run.
   USERID's value is the host's, so only its shape is printed. */
say qualify('/a/../b/c.txt')
say qualify('/x//y/./z/')
say '[' || qualify('') || ']'
say qualify('rel.txt') == qualify('./rel.txt')
say length(userid()) > 0
say 'setlocal' setlocal()
call value 'P7_LOCAL_VAR', 'inner', 'ENVIRONMENT'
say '[' || value('P7_LOCAL_VAR', , 'ENVIRONMENT') || ']'
say 'endlocal' endlocal()
/* A restore puts back what it saved and removes nothing added since. There is
   deliberately no second ENDLOCAL: one past the outstanding SETLOCALs is
   corpus/oracle-crashes.txt entry 10a, and this crate's answer for it is a
   licensed divergence rather than a differential. */
say '[' || value('P7_LOCAL_VAR', , 'ENVIRONMENT') || ']'
