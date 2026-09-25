say 'try'
signal on syntax
say .context~package~loadLibrary('forgever')
exit
syntax: say 'code' condition('o')~code; say 'again' .context~package~loadLibrary('forgever'); say 'ext' .Routine~loadExternalRoutine('r', 'LIBRARY forgever Stash')
