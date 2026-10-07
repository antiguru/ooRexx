/* OPTIONS takes any expression: a variable, a call, an object without a
   string value, and one whose string value comes from MAKESTRING, which is
   traced as the result. */
x = 'NOVALUE'
options x
options f()
options .array~new
say 'ok' n
trace r
options x 'EXMODE'
options .K
trace o
exit
f: n = 'called'; return 'NOEXMODE'
::class K
::method makeString class
  return 'NOVALUE'
