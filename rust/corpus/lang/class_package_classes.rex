/* Task 21: the package object's own class tables. `::CLASS ... PUBLIC` files
   a class in both, a plain `::CLASS` in one, and `~addClass`/`~addPublicClass`
   are the same split made by hand. Each answers the package itself, and
   `~publicClasses` answers a fresh table every time -- the identity row below
   is `0` for that reason and is the one row that says so. */
p = .context~package
say p~publicClasses~class
say p~publicClasses["K"]
say p~publicClasses["PRIV"]
say (p~addClass("zz", .K)~identityHash == p~identityHash)
say p~publicClasses["ZZ"]
say (p~addPublicClass("yy", .K)~identityHash == p~identityHash)
say p~publicClasses["YY"]
say (p~publicClasses~identityHash == p~publicClasses~identityHash)

::class K public
::class Priv
