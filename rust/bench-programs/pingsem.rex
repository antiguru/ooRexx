/* Semaphore post/wait: two activities hand a turn back and forth through two
   event semaphores, n times each way. */
n = 100000
ping = .eventSemaphore~new
pong = .eventSemaphore~new
.player~new~start('play', ping, pong, n)
do i = 1 to n
    ping~post
    pong~wait
    pong~reset
end
say 'done'

::class player
::method play
    use arg ping, pong, n
    do i = 1 to n
        ping~wait
        ping~reset
        pong~post
    end
