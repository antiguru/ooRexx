/* EventSemaphore in one activity: a post stays until a reset, a wait on a
   posted semaphore answers at once, a timeout of zero answers the post, and
   an UNINIT leaves the post as it was. */
e = .EventSemaphore~new
say 'new' e~isPosted e~wait(0) e~wait(0.0001) e~wait(0.002)
say 'timespan' e~wait(.TimeSpan~fromMicroseconds(2000))
e~post
e~post
say 'posted' e~isPosted e~wait e~wait(-5) e~wait(1e10) e~wait(0) e~wait(5)
e~reset
say 'reset' e~isPosted e~wait(0)
signal on syntax name notNumber
say e~wait('abc')
notNumber:
say 'error' condition('o')~code condition('o')~message
signal on syntax name nilTimeout
say e~wait(.nil)
nilTimeout:
say 'error' condition('o')~code condition('o')~message
e~post
e~uninit
say 'after uninit' e~isPosted e~wait(0)
e~reset
say 'reset again' e~isPosted
