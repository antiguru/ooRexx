c = .Bag~of(1, 1, 1)
say c~hasItem(1, .nil)
say c~hasItem(1, 1)
say c~hasItem(1, 2)
q = .queue~of('Mike', 'David', 'Tom', 'David')
say q~removeItem('David') q~items q[1] q[2] q[3]
