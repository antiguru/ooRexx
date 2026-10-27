/* a stem subclass instance's own rendering, and a dynamic subclass */
t = .mystem~new
say '[' || t || ']'
say length(t)
say t~string
u = .stem~subclass('dyn')~new
say '[' || u || ']'
u[] = 'uu'
say u
v = .stem~new('V.')
say v
say 'end'
::class mystem subclass stem
