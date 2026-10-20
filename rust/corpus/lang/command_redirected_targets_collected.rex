/* A parked redirected command's targets that nothing else holds stay alive
   while another activity allocates enough to collect. */
msg = .Job~new~start('run')
churned = 0
do until msg~completed
  junk = .array~new(1000)
  do k = 1 to 50
    junk[k] = .array~of(k, k || 'x', .directory~new)
  end
  churned += 1
end
say 'churned' (churned > 0)
say 'rc' msg~result
exit

::class Job
::method run
  address system 'echo a; echo b; sleep 0.5; echo c >&2' -
    with output using (.Sink~new~~tag('out')) error using (.Sink~new~~tag('err'))
  return rc

::class Sink subclass Array
::method tag
  expose tag
  use arg tag
::method append
  expose tag
  use arg line
  say tag line
  return self~append:super(line)
