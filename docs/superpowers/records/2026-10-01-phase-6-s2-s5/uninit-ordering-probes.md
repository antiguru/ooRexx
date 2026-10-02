# UNINIT ordering probes (Phase 6 S2-S5 Task 9)

The oracle probes behind `phase-6-gate.md` `## S2`. Each runs from a fresh empty directory under
the oracle wrapper; the outputs are there.

## u1

```rexx
-- live object at end; a started activity still running when main ends
o = .k~new('live')
.w~new~start('run')
say 'main end'
::class k
::method init
  expose n
  use arg n
::method uninit
  expose n
  say 'uninit' n
::class w
::method run
  call SysSleep 0.3
  say 'activity end'
```

## u2

```rexx
-- object dropped in a started activity; no forced collection
m = .w~new~start('run')
m~wait
say 'main end'
::class k
::method init
  expose n
  use arg n
::method uninit
  expose n
  say 'uninit' n
::class w
::method run
  o = .k~new('dropped')
  drop o
  say 'activity end'
```

## u3

```rexx
-- object dropped in a started activity; GC('force') in that activity
m = .w~new~start('run')
m~wait
say 'main end'
::class k
::method init
  expose n
  use arg n
::method uninit
  expose n
  say 'uninit' n .context~thread
::class w
::method run
  o = .k~new('dropped')
  drop o
  call gc 'force'
  say 'activity end'
```

## u4

```rexx
-- object dropped in a started activity; collection forced by allocation, then more returns
m = .w~new~start('run')
m~wait
say 'main end'
::class k
::method init
  expose n
  use arg n
::method uninit
  expose n
  say 'uninit' n
::class w
::method run
  o = .k~new('dropped')
  drop o
  do i = 1 to 200000
    s = .array~new(10)
  end
  say 'activity loop done'
  call f
  say 'activity end'
  return
f: return
```

## u5

```rexx
o = .k~new
say 'main end'
::class k
::method uninit
  say 'uninit before reply'
  reply
  say 'uninit after reply'
```

## t3

```rexx
o = .k~new
say 'main end'
exit 7
::class k
::method uninit
  say 'uninit starts'
  .z~new~start('go')
  say 'uninit after start'
::class z
::method go
  call SysSleep 0.2
  say 'z go on' .context~thread
::method bad
  call SysSleep 0.2
  say 'z fails'
  x = 1/0
```

## t7

```rexx
o = .k~new
say 'main end'
::class k
::method uninit
  say 'uninit starts a poller'
  .z~new~start('go')
::class z
::method go
  do forever
    call SysSleep 1
  end
```
