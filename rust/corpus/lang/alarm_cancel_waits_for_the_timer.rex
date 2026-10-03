/* A cancel sent at once waits until the replied activity has started the
   timer, then cancels it. */
t = .target~new
a = .alarm~new(5, t)
a~cancel
say 'alarm' a~triggered a~canceled a~cancelled
::class target inherit AlarmNotification
::method triggered
  say 'triggered'
::method cancel
  use arg alarm
  say 'cancel notified' alarm~canceled
