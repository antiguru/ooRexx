/* gate_variants.rex -- reaches the AST variants that no other corpus program
   and no file under samples/ constructs, so the phase 3 gate's enumeration
   test (rust/crates/rexx-parse/tests/variants.rs) can assert every variant is
   built at least once. Each block names the variants it exists for.
   Everything with a runtime effect is behind "if 0 then" or inside a routine
   or method that is never called, because loading this file as a package runs
   its prolog, and the gate's SOURCELINE driver does exactly that. */

/* InstructionKind::Queue, Trace::Skip, Trace::Default */
if 0 then queue "never queued"
if 0 then trace 5
if 0 then trace

/* InstructionKind::Options */
if 0 then options "GATE_VARIANTS"

/* Signal::Value, resolved at run time and never run; the label it would
   resolve to is below. */
if 0 then signal value "AFTER_VALUE"
after_value:

/* ExprKind::QualifiedCall, ExprKind::ClassResolver, Call::Qualified */
if 0 then x = gate_ns:helper(1)
if 0 then y = gate_ns:someclass
if 0 then call gate_ns:helper 2

/* Call::Trap */
if 0 then call on user gatecond name after_value

/* ExprKind::VariableReference */
if 0 then z = >refvar

/* LoopKind::With */
if 0 then do with index i item v over .nil
  nop
end
exit

/* ParseSource::LineIn lives here because running "parse linein" would block
   on stdin, and this routine is never called. */
::routine never_called
  parse linein first_line
  return first_line

/* Use::Local is only legal inside a method, and a method body never runs at
   package install time. */
::class gate_variants_class
::method gate_method
  use local a b
  return

/* DirectiveKind::Annotate: PACKAGE always exists as a target. */
::annotate package gate_purpose "variant coverage"
