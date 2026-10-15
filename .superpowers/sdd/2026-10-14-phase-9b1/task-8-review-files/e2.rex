m = .k~method('BODY2')~copy~~setUnguarded
say m~isGuarded
e = .object~enhanced(.directory~of(('BAR', m)))
say e~bar
say e~instanceMethod('BAR')~isGuarded (e~instanceMethod('BAR') == m)
exit
::class k
::method body2
  return 'bar' .context~executable~isGuarded (.context~executable == arg(1))
