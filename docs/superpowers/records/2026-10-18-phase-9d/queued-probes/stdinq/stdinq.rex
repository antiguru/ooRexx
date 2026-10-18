s = .stream~new('STDIN')
say 'type' s~query('streamtype')
say 'handle' s~query('handle')
say 'chars' s~chars
say 'lines' s~lines
say 'open' s~open('read')
say 'exists' s~query('exists')
say 'qualify' s~qualify
