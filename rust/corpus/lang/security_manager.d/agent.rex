call on error name errtrap
say 'filespec' filespec('N', '/tmp/zz.txt')
say 'lines   ' lines('data.txt')
say 'dotname ' .SOMENAME
object = .Guarded~new
say 'method  ' object~kept
'echo command'
say 'rc      ' rc '.rs' .rs
loaded = .context~package~loadPackage('lib1.rex')
say 'requires' (right(loaded~name, 8) == 'lib1.rex')
exit 0

errtrap:
say 'trapped ' rc
return

::class Guarded
::method kept protected
  return 'kept-ran'
