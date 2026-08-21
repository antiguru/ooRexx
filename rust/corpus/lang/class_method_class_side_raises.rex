/* ~method for a name that is the class's own CLASS method raises, untrapped,
 * so the traceback bytes are compared rather than only the condition number.
 *
 * The pair with class_method_own_dictionary.rex, whose untrapped send is the
 * donated-name shape: a build reading the class behaviour instead of the
 * instance dictionary answers this one and raises that one, so neither
 * program alone separates the two dictionaries.
 *
 * The frame line is Task 6's, and this refusal is owed it by the rule
 * blame_native_method states: the oracle reached METHOD's body by an explicit
 * message send. rc 159.
 */

say .K~method('M')

::CLASS K
::METHOD m CLASS
  return 'class side'
