o = .array~of(1,2)
m = .Message~new(o,'ITEMS')
m~notify(.bad~new)
signal on syntax name s1
m~send
say 'send no raise'
s1: signal off syntax
say 'completed' m~completed
m~notify(.good~new)
say 'after second notify'
w = .WeakReference~new(m)
m = .nil
do 30; z = copies('x',100000); end
do i = 1 to 20
  n = .Message~new(o,'ITEMS')
  n~send
  n~notify(.good~new)
end
say 'loop done'
::class bad inherit MessageNotification
::method messageComplete
  raise syntax 93.900 array('bad party')
::class good inherit MessageNotification
::method messageComplete
  say 'good notified'
