/* A ::CLASS directive in a file reached by an external CALL resolves its
   SUBCLASS through that file's package: the caller's class is not visible. */
signal on syntax name s
call 'lib2.rex'
say 'loaded'
exit
s: say 'error' condition('O')~code; exit
::class hiddenx
