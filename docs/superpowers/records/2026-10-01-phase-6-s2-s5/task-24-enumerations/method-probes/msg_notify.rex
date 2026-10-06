m = .message~new("abc", "LENGTH"); m~send; m2 = .message~new("abcd", "LENGTH"); m~notify(m2); say "ok" m2~completed m2~result
