m = .message~new("abc", "LENGTH"); m~start; say "ok" m~result; n = .message~new("x","LENGTH"); n~start("abcd"); say "ok" n~result
