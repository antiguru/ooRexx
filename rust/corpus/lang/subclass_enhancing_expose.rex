/* Class methods from source text expose the class object's variables, and an
 * array source takes its arguments. */
c = .object~subclass('K', .class, .directory~of(('M', 'expose v; v = 7; return v'), ('N', 'expose v; return v')))
say c~m c~n
d = .object~mixinClass('Q', .class, .directory~of(('M', ('use arg a', 'return a * 2'))))
say d~m(21)
