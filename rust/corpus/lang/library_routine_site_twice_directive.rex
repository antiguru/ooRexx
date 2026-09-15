/* A ::ROUTINE bound to a library entry, called a second time from one
   expression call site, is reported under the directive's name. */
do a over .array~of(4, 'x')
  say sq(a)
end
::requires 'pk.cls'
