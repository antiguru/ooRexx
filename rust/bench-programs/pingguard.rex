/* GUARD WHEN handoff: two activities pass a turn through one object's
   variable, each waiting in GUARD ON WHEN for its own turn, n times each. */
n = 100000
t = .table~new
t~start('pong', n)
t~ping(n)
say 'done'

::class table
::method init
    expose turn
    turn = 0
::method ping
    expose turn
    use arg n
    do i = 1 to n
        guard on when turn = 0
        turn = 1
        guard off
    end
::method pong
    expose turn
    use arg n
    do i = 1 to n
        guard on when turn = 1
        turn = 0
        guard off
    end
