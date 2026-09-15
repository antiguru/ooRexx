/* A merged library routine called a second time from one expression call site
   is reported under the name the call wrote. */
do a over .array~of(4, 'x')
  say RxCalcSqrt(a)
end
::requires 'pk.cls'
