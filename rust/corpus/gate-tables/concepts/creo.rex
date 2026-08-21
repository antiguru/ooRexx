/* provide.xml `creo`: an object requiring initialization defines INIT, the
   class object runs it after the object is created with the arguments given
   to NEW, and a class with more than one INIT forwards the INIT message up
   the hierarchy. */
asav = .savings~new(1000.00, 6.25)
say 'type' asav~type
say 'balance' asav~balance
say 'rate' asav~rate

::class account
::method init
  expose balance
  use arg balance
::method type
  return "an account"
::attribute balance get

::class savings subclass account
::method init
  expose interest_rate
  use arg balance, interest_rate
  self~init:super(balance)
::method type
  return "a savings account"
::method rate
  expose interest_rate
  return interest_rate
