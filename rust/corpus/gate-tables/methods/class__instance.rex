/* Table C method rows: Class, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 93.901 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Class~new
say 'instance' o~hasMethod("=")
say 'instance' o~hasMethod("==")
say 'instance' o~hasMethod("<>")
say 'instance' o~hasMethod("><")
say 'instance' o~hasMethod("\=")
say 'instance' o~hasMethod("\==")
say 'instance' o~hasMethod("activate")
say 'instance' o~hasMethod("annotation")
say 'instance' o~hasMethod("annotations")
say 'instance' o~hasMethod("baseClass")
say 'instance' o~hasMethod("defaultName")
say 'instance' o~hasMethod("define")
say 'instance' o~hasMethod("defineMethods")
say 'instance' o~hasMethod("delete")
say 'instance' o~hasMethod("enhanced")
say 'instance' o~hasMethod("id")
say 'instance' o~hasMethod("inherit")
say 'instance' o~hasMethod("isAbstract")
say 'instance' o~hasMethod("isMetaclass")
say 'instance' o~hasMethod("isSubclassOf")
say 'instance' o~hasMethod("metaClass")
say 'instance' o~hasMethod("method")
say 'instance' o~hasMethod("methods")
say 'instance' o~hasMethod("mixinClass")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("queryMixinClass")
say 'instance' o~hasMethod("subclass")
say 'instance' o~hasMethod("subclasses")
say 'instance' o~hasMethod("superClass")
say 'instance' o~hasMethod("superClasses")
say 'instance' o~hasMethod("uninherit")
