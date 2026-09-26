t = .T~new
say t~is('abc') t~is(12) t~is('12') t~is(1+1) t~is(1/3) t~is(2**70) t~is(.array~new) t~is(.nil) t~is(.string~new('x')) t~is(3.5) t~is(10000000000)
say t~o2s(12) t~o2s(.array~of(1,2)) t~o2s(.object~new) t~o2s(1/3)
::class sub subclass string
::class T
::method is external "LIBRARY orxmethod TestIsString"
::method o2s external "LIBRARY orxmethod TestObjectToString"
