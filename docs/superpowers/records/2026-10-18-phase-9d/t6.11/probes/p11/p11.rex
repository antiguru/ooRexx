a = names(.Object~methods(.nil))
b = names(.WaterVehicle~methods)
v = names(.Vehicle~methods(.nil))
w = names(.WaterVehicle~methods(.nil))
say 'o' a~items 'all' b~items 'v' v~items 'w' w~items
say 'o-all' a~difference(b)~makearray~sort~toString('L', ' ')
say 'all-o-v-w' b~difference(a)~difference(v)~difference(w)~makearray~sort~toString('L', ' ')
a = items(.Object~methods(.nil))
b = items(.WaterVehicle~methods)
say 'items o-all' a~difference(b)~items
exit
names: procedure
  use arg s
  bag = .bag~new
  do while s~available; bag~put(s~index); s~next; end
  return bag
items: procedure
  use arg s
  bag = .bag~new
  do while s~available; bag~put(s~item); s~next; end
  return bag
::CLASS  Vehicle                                   -- base class
::METHOD name    ATTRIBUTE
::METHOD INIT
  self~name = ARG(1)

::METHOD show_off
  return "Vehicle's SHOW_OFF method"

::CLASS  RoadVehicle   MIXINCLASS Vehicle
::METHOD drive
  RETURN self~name": 'I drive now...'"

::CLASS  WaterVehicle     MIXINCLASS Vehicle
::METHOD swim
  RETURN self~name": 'I swim now...'"

::CLASS AmphibianVehicle SUBCLASS RoadVehicle INHERIT WaterVehicle
::METHOD show_off
  RETURN self~drive self~swim

::class Transportation mixinclass object
