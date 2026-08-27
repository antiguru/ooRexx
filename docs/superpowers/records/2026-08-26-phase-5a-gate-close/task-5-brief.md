## Task 5: `~define` with source text

**Goal.** `corpus/gate-tables/concepts/methna.rex` agrees.

**Measured.** Oracle **rc 0**, four lines: `id COST`, then `The Method class` three times for
`~method("%")`, `~method("TYPE")` and `~method("type")`. Crate: rc 120,
`a method built from source text is not implemented`.

**What is already there, probed separately.** `~define` with a **method object** works today;
`~method(...)~class` answers `The Method class`. What is missing is compiling a method body from a
string outside a `::METHOD` directive.

**Build.** `MethodClass::newMethodObject` compiles anything that is not already a method object
(`classes/MethodClass.cpp:457`-`:486`). The row also pins that **the dictionary key is the upcased
name** while a quoted name keeps its spelling as the method's own name: `~method("TYPE")` and
`~method("type")` both answer for a name defined as `"type"`, and `"%"` is a name no symbol could
hold.

**Say what a compiled body can and cannot do in this phase**, since the body is real Rexx and
reaches the interpreter: the row's own bodies are `return` of a literal. A body that reaches an
unbuilt mechanism must refuse loudly rather than answer wrongly.

**Done when** the row agrees on both engines, the upcasing pair is pinned by a corpus row of this
task's own, and a control is recorded: keeping the as-written spelling as the dictionary key makes
`~method("TYPE")` raise where the oracle answers, and the row reddens.

---


## Controller addendum, measured at `637d0dd64` before dispatch

**Treat the brief's "what is already there" paragraph as a claim to re-measure, not a premise.** It
has been wrong twice on this plan, both times in the direction of making the task look smaller.
Correct the plan file where you find it wrong, not this brief and not your report.

**Measured on the oracle, past the row.** These are the facts a body-from-source build has to get
right that the row does not reach:

```
.cost~define("upper", 'return "U"')
say 'a' .cost~new~upper              ->  U
say 'b' .cost~method("UPPER")~scope~id  ->  COST
say 'c' .cost~method("upper")~class~id  ->  Method
d = .method~new("q", 'return 1')
say 'd' d~scope                      ->  The NIL object
```

* **`~define` returns no result.** `say .cost~define(...)` is oracle **rc 165**, `Error 91.999:
  Message "DEFINE" did not return a result.` So it cannot sit in an expression.
* **A body may be several lines**, passed as an array of strings, and `~source` reads them back:
  ```
  b = .array~new; b~append('use arg n'); b~append('return n * 2')
  .cost~define("dbl", b)
  say .cost~new~dbl(21)                        ->  42
  say .cost~method("DBL")~source~makeString('L','|') ->  use arg n|return n * 2
  ```
  A single string body is the one-line case of the same thing. Decide and record whether this task
  builds the array form or refuses it loudly; do not answer it wrongly.
* **A body that does not parse fails at `~define` time, not at send time**, and the diagnostic names
  **the method** as the program:
  ```
  .cost~define("bad", 'this is not rexx +++')
  ```
  oracle **rc 221**, stderr:
  ```
         *-* Compiled method "DEFINE" with scope "Class".
       1 *-* .cost~define("bad", 'this is not rexx +++')
  Error 35 running bad line 1:  Invalid expression.
  Error 35.901:  Prefix operator "+" is not followed by an expression term.
  ```
  `running bad` -- the method's name where a file path normally goes -- and `line 1` counted inside
  the body. Pin this; it is easy to get right by accident and easy to get wrong invisibly.

**Task 2 built `Method~scope`.** The `~scope~id` line above is the interaction: a method compiled
from source text and defined into a class must answer that class, and `.method~new` on its own must
answer `.nil`. Both are measured above.

**The hazard, restated.** Every task on this plan turns a loud refusal into an answer, which is
exactly where a silent wrong answer at rc 0 is introduced, and no gate sees one. Probe past the row.
