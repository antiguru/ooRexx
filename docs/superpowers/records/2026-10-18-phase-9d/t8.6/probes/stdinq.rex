s = .stream~new('STDIN')
say 'st' s~query('streamtype')
say 'h' s~query('handle')
say 'c' s~chars
say 'l' s~lines
say 'ex' s~query('exists')
say 'q' s~qualify
say 'state' s~state s~description
say 'size' s~query('size')
say 'open' s~open('read')
say 'st' s~query('streamtype')
say 'h' s~query('handle')
say 'c' s~chars
say 'l' s~lines
say 'ex' s~query('exists')
say 'q' s~qualify
say 'state' s~state s~description
say 'size' s~query('size')
say 'pos' s~query('position')
say 'close' s~close
s = .stream~new('STDIN:')
say 'st' s~query('streamtype') s~qualify s~query('exists') s~query('handle')
say 'bn' stream('STDIN','c','query exists') stream('STDIN','c','query streamtype') stream('STDIN','c','query handle')
