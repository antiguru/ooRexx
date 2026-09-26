t = .T~new
say '['t~data(1/3)']' t~len(1/3) t~get(2/3, 1, 4) t~up(1/3)
::class T
::method len external "LIBRARY orxmethod TestStringLength"
::method data external "LIBRARY orxmethod TestStringData"
::method get external "LIBRARY orxmethod TestStringGet"
::method up external "LIBRARY orxmethod TestStringUpper"
