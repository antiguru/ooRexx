s = .WaterVehicle~methods
do while s~available; say s~index s~item~scope~id; s~next; end
s = .Object~methods(.nil)
do while s~available; m = s~item; t = .WaterVehicle~methods; f = 0; do while t~available; if t~item == m then f = 1; t~next; end; if \f then say 'missing' s~index; s~next; end
::CLASS  Vehicle                                   -- base class
::ATTRIBUTE name
::METHOD init
  self~name = arg(1)
::CLASS  WaterVehicle     MIXINCLASS Vehicle
::METHOD swim
