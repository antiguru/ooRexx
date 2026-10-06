m = .message~new("abc", "LENGTH"); r = m~reply; say "ok" r~result m~completed; n = .message~new("x", "LENGTH"); r2 = n~reply("abcd"); say "ok" r2~result
