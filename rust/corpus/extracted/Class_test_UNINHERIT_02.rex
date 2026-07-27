/* extracted from Class::test_UNINHERIT_02 */
::routine main public
   -- create a base test_class
   vehicle_Name="RGF_VEHICLE"
   rgf_vehicle=.object~subclass(vehicle_Name)

   -- create RoadVehicle
   road_Vehicle_Name="RGF_ROADVEHICLE"
   rgf_road_vehicle=rgf_vehicle~mixinclass(road_Vehicle_Name)

   -- create WaterVehicle
   water_Vehicle_Name="RGF_WaterVehicle"
   rgf_water_vehicle=rgf_vehicle~mixinclass(water_Vehicle_Name)

   -- create AmphibianVehicle
   amphibian_Vehicle_Name="RgF_AmPhIbIaNvEhIcLe"
   rgf_amphibian_vehicle=rgf_road_vehicle~subclass(amphibian_Vehicle_Name)
   rgf_amphibian_vehicle~inherit(rgf_water_vehicle)

   rgf_amphibian_vehicle~uninherit(rgf_water_vehicle)

   self~expectSyntax(98.945)
   rgf_amphibian_vehicle~uninherit(rgf_water_vehicle)

::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
