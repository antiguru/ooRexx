do i = 1 to 3
  say 'function' i RxCalcSqrt(helper1(16))
end
do i = 1 to 3
  call RxCalcCos helper2(0)
  say 'call' i result
end
do i = 1 to 3
  x = RxCalcExp(helper3(0))
  say 'assignment' i x
end
