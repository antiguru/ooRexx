/* A cancel ends a timer's wait during its whole days, once the timer has
   started: an Alarm two days out and a Ticker of a day. */
t = .target~new
a = .alarm~new(2 * 86400, t)
call SysSleep 0.2
a~cancel
say 'alarm' a~triggered a~canceled
k = .ticker~new(86400, t)
call SysSleep 0.2
k~cancel
say 'ticker' k~canceled
::class target inherit AlarmNotification
::method triggered
  use arg timer
  say 'triggered' timer~class~id
::method cancel
  use arg timer
  say 'cancel notified' timer~class~id
