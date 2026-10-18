/* IdentityTable TEST_AT01, TEST_[] and TEST_HASINDEX: every assertion that
   does not ask whether two separately built equal strings are one object
   (Deviation 4), and those questions printed, not asserted. */
indexes = .array~of("22", "2"||"2", .object~new, .array~new)
values = .array~of(1, 2, 3, 4)
coll = .IdentityTable~new
do i = 1 to indexes~size
  coll[indexes[i]] = values[i]
end
ok = 1
do i = 1 to indexes~size
  if coll~at(indexes[i]) \== values[i] then ok = 0
  if coll[indexes[i]] \== values[i] then ok = 0
  if \coll~hasIndex(indexes[i]) then ok = 0
end
say 'at, [] and hasIndex of each held index' ok
say 'at(.object~new) is .nil' (coll~at(.object~new) == .nil)
say '[.object~new] is .nil' (coll[.object~new] == .nil)
say 'Deviation 4, unasserted: at("2"||"2") is .nil' (coll~at("2"||"2") == .nil)
say 'Deviation 4, unasserted: ["2"||"2"] is .nil' (coll["2"||"2"] == .nil)
say 'Deviation 4, unasserted: hasIndex(substr) is 0' (coll~hasIndex("1223"~substr(2,2)) == 0)
