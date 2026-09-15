do i = 1 to 2
  say 'parent' i RxCalcPower(2, 3)
  if i = 1 then p = .context~package~loadPackage('pub.cls')
end
say 'fresh' RxCalcPower(2, 3)
