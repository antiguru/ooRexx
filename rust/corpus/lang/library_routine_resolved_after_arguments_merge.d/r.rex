.local~loaded = 0
do i = 1 to 3
  say 'loadPackage' i RxCalcSqrt(load(16))
end
exit
load:
  .local~loaded = .local~loaded + 1
  if .local~loaded = 1 then p = .context~package~loadPackage('pub.cls')
  return arg(1)
