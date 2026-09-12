/* `cd`, `set`, `unset` and `export` are run against the interpreter's own
   directory and environment rather than in a child, so their effect outlives
   the command. Nothing here prints an absolute path: the directory both
   engines run in is the harness's own, and the answers below are about
   whether it moved rather than about where it is. */

d0 = directory()
made = .File~new('sub')~makeDir
'cd sub'
say 'cd        rc=' rc 'moved=' (directory() \== d0)
'cd ..'
say 'back      rc=' rc 'same=' (directory() == d0)

/* A `chdir` that fails is still handled here rather than handed to a shell:
   the return code is the `errno`, and nothing reaches standard error. */
'cd nosuchdir'
say 'bad cd    rc=' rc 'rs=' .rs 'same=' (directory() == d0)

/* An unquoted `;` means a shell has to see it, so this one is not handled
   internally and the directory does not move. */
'cd sub ; true'
say 'guarded   rc=' rc 'same=' (directory() == d0)

'export ZZTOP=hello'
say 'export    rc=' rc 'value=' value('ZZTOP',,'ENVIRONMENT')
'set ZZSET=two'
say 'set       rc=' rc 'value=' value('ZZSET',,'ENVIRONMENT')

/* The value half is expanded against the environment before it is stored. */
'export ZZEXP=$ZZSET-tail'
say 'expanded  rc=' rc 'value=' value('ZZEXP',,'ENVIRONMENT')

'unset ZZTOP'
say 'unset     rc=' rc 'value=[' || value('ZZTOP',,'ENVIRONMENT') || ']'

/* A child sees the environment and the directory the interpreter is holding,
   not the ones the process started with. */
'cd sub'
'printf %s-%s $ZZSET done'
say ''
