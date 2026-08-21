/* provide.xml `methna`: a method name can be any string, a name holding
   characters that are not allowed in a symbol must be quoted, and the
   language processor matches a message name against the method name in
   uppercase -- including for the names it adds. */
.cost~define("%", 'return "percent"')
.cost~define("type", 'return "a cost"')
say 'id' .cost~id
say 'operator-name' .cost~method("%")~class
say 'added-lowercase' .cost~method("TYPE")~class
say 'matched-in-uppercase' .cost~method("type")~class

::class cost
