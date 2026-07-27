numeric digits 9
numeric fuzz 0
say "fuzz 0:" ( 1.00000000 = 1.00000001 )
numeric fuzz 1
say "fuzz 1:" ( 1.00000000 = 1.00000001 )
say "fuzz():" fuzz()
numeric fuzz 5
say "fuzz 5:" ( 1.0000 = 1.0001 )
numeric fuzz 0
say "restored:" fuzz()
