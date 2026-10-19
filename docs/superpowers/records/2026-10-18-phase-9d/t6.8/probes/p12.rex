m = .Method~new("", "return 1")
say '['m~package~name']'
m = .Method~new("mm", "return 1")
say '['m~package~name']'
r = .Routine~new("", "return 1")
say '['r~package~name']'
r = .Routine~new("rr", "return 1")
say '['r~package~name']'
