do d = 1 to 12
  numeric digits d
  say d":" (1/3) (2/3) (10/3) (1/7)
end
numeric digits 40
say "40:" (1/3)
say "40:" (2/7)
numeric digits 9
/* rounding is round-half-up on the discarded part */
say 0.5 + 0 , 1.5 + 0 , 2.5 + 0 , -0.5 + 0
say 1.0000000049 + 0
say 1.0000000050 + 0
say 1.0000000051 + 0
