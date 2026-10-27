/* acyclic, forwarded messages and operators through a stem default */
s. = 5
r = >s.
say r~request('STRING')
say r~string
say s.~request('STRING')
say s.~string
say s.~foo
say 'end'
::class x
