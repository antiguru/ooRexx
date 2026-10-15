/* For each way a program makes a class: does a WeakReference to a dropped one
   clear while the loop makes more of them? Prints the iteration it cleared at. */
call try 'subclass', "c = .object~subclass('t')"
call try 'class new', "c = .class~new('t2')"
call try 'mixinclass', "c = .object~mixinclass('m')"
call try 'metaclass', "c = .class~subclass('meta')"
call try 'subclass of subclass', "c = .object~subclass('a')~subclass('b')"
call try 'class uninit', "c = .object~subclass('u'); c~define('UNINIT', 'nop')"
call try 'package new', "c = .Package~new('p', ('::class c', '::method a'))~classes['C']"
call try 'requires', "c = .Package~new('pr', '::requires ''reqc.rex''')~importedPackages[1]~classes['RC']"
call try 'routine new', "c = .Routine~new('r', ('return 1', '::class rcl'))~package~classes['RCL']"
call try 'with instance dropped', "c = .object~subclass('wi'); i = c~new"
call try 'class new subclass', "c = .class~new('t3')~subclass('t4')"
exit
try: procedure
  parse arg tag, make
  interpret make
  w = .WeakReference~new(c)
  drop c i
  do n = 1 to 20000 until w~value == .nil
    interpret make
    drop c i
  end
  say tag 'cleared:' (w~value == .nil)
  return
