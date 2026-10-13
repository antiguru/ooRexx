say 'type' stream('HANDLE:0','C','query streamtype')
say 'state' stream('HANDLE:0','S')
say 'desc' stream('HANDLE:0','D')
say 'exists [' || stream('HANDLE:0','C','query exists') || ']'
say 'handle [' || stream('HANDLE:0','C','query handle') || ']'
say 'l1' linein('HANDLE:0')
say 'state' stream('HANDLE:0','S')
say 'type' stream('HANDLE:0','C','query streamtype')
say 'handle [' || stream('HANDLE:0','C','query handle') || ']'
say 'exists [' || stream('HANDLE:0','C','query exists') || ']'
say 'c' charin('HANDLE:0')
say 'lines' lines('HANDLE:0')
say 'l2' linein('HANDLE:0')
say 'l3' linein('HANDLE:0')
say 'lines' lines('HANDLE:0')
say 'chars' chars('HANDLE:0')
say 'eof [' || linein('HANDLE:0') || ']'
say 'state' stream('HANDLE:0','S') stream('HANDLE:0','D')
say 'close' stream('HANDLE:0','C','close')
say 'state' stream('HANDLE:0','S')
