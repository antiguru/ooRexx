t = .test~new
t~put('m1', .methods~fm, 'float')
t~m1
::method fm
  say .context~executable~scope
::class test
::method put
  use arg name, meth, scope
  self~setMethod(name, meth, scope)
