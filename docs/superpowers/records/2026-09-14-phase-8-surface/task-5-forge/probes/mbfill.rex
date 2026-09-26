c = .MutableBuffer~new('abc', 4000)~copy
say MBFill(c)
say c~length
exit
::requires 'reach' LIBRARY
