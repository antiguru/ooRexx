/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

use super::{Interp, ProgramId, form_name, parse_program, run_program};
use rexx_parse::{DirectiveKind, Expr, ExprKind, Operator, PrefixOp, Program};
use std::rc::Rc;

/// An empty clause span is what tells a directive this crate synthesised
/// from one a program wrote, and three walks skip on it --
/// [`Interp::install_directives`], `class_members` and
/// `environment.rs`'s `package_table_entries`.
#[test]
fn no_written_directive_has_an_empty_clause_span() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let mut directives = 0usize;
    let mut directories = vec![corpus];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable corpus directory") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rex") {
                continue;
            }
            let bytes = std::fs::read(&path).expect("a readable corpus program");
            let Ok(program) = parse_program(bytes) else {
                continue;
            };
            for directive in &program.directives {
                assert!(
                    !directive.clause_span.is_empty(),
                    "{} has a written directive whose clause span is empty, so the three \
                     walks that skip on an empty span would stop installing it",
                    path.display()
                );
                directives += 1;
            }
        }
    }
    assert!(
        directives > 100,
        "the walk found {directives} directives, which is too few for the corpus to have \
         been read at all"
    );
}

/// The path these tests report programs under.
const TEST_PATH: &str = "/nonexistent/lib-test-program.rex";

/// A program with a body per directive kind that carries one, which is what
/// [`super::render_ir`] walks.
const EVERY_BODY: &[u8] = b"\
say 1
::class k
::method m
  return 1
::attribute a get
  return 3
::routine r
  return 2
";

/// [`super::render_ir`] renders every body a program has, each under its
/// own heading -- not the main body alone.
#[test]
fn the_ir_render_covers_every_body_and_only_the_ones_that_exist() {
    let rendered = super::render_ir(EVERY_BODY.to_vec(), b"n").expect("the program parses");
    let headings: Vec<&str> = rendered
        .lines()
        .filter(|line| line.starts_with("==="))
        .collect();
    assert_eq!(
        headings,
        vec![
            "=== main ===",
            "=== ::METHOD ===",
            "=== ::ATTRIBUTE ===",
            "=== ::ROUTINE ==="
        ],
        "the render walked the wrong set of bodies\n{rendered}"
    );
    // Each body's own ops are there, not just its heading: the three
    // directive bodies each return a literal, so each owes a load and a
    // `Return`, and the main body owes a `Say`.
    assert_eq!(rendered.matches("Return").count(), 3, "{rendered}");
    assert_eq!(rendered.matches("Say").count(), 1, "{rendered}");
}

/// **The `TRACE` setting is an input to compilation, not a display
/// option** (D23), and the render shows the difference.
#[test]
fn the_ir_render_compiles_under_the_setting_it_is_given() {
    let untraced = super::render_ir(EVERY_BODY.to_vec(), b"n").expect("the program parses");
    let traced = super::render_ir(EVERY_BODY.to_vec(), b"r").expect("the program parses");
    assert!(
        !untraced.contains("TraceClause"),
        "a chunk compiled under N carries a clause echo op\n{untraced}"
    );
    assert!(
        traced.contains("TraceClause"),
        "a chunk compiled under R carries no clause echo op\n{traced}"
    );
}

/// **A clause-opening op carries the source clause it stands for, and an
/// op inside a region does not.**
#[test]
fn the_ir_render_names_the_clause_a_region_opens_and_not_its_inner_ops() {
    // `DROP` delegates and `SAY` has compiled operands, so this program
    // has a region of each shape: one holding a single `Exec`, one holding
    // the ops that compute the value.
    let rendered = super::render_ir(b"drop zn\nsay 'x'\n".to_vec(), b"n").expect("parses");
    let lines: Vec<&str> = rendered.lines().collect();
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("0: Clause") && line.ends_with("; 1: drop zn")),
        "the Clause op does not name the clause it opens\n{rendered}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("2: Clause") && line.ends_with("; 2: say 'x'")),
        "the second Clause op does not name the clause it opens\n{rendered}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("1: Exec") && !line.contains(';')),
        "the Exec op inside a region repeats its region's clause\n{rendered}"
    );
    assert!(
        lines
            .iter()
            .filter(|line| line.starts_with("3: ") || line.starts_with("4: "))
            .all(|line| !line.contains(';')),
        "an op inside a region repeats its region's clause\n{rendered}"
    );
}

/// A setting that is not a `TRACE` setting is reported, rather than
/// silently compiling under some other one.
#[test]
fn the_ir_render_refuses_a_setting_that_is_not_one() {
    let refused = super::render_ir(EVERY_BODY.to_vec(), b"zz");
    assert!(
        refused.is_err_and(|report| report.contains("not a TRACE setting")),
        "an unusable TRACE setting was accepted"
    );
}

fn literal() -> Expr {
    Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1)
}

fn nest(depth: usize) -> Expr {
    let mut node = literal();
    for _ in 0..depth {
        node = Expr::new(
            ExprKind::Binary {
                op: Operator::Plus,
                left: Box::new(node),
                right: Box::new(literal()),
            },
            0..1,
        );
    }
    node
}

/// `Loud::expression`'s size contract, tested on the two arms that can
/// break it.
#[test]
fn the_two_formatting_arms_do_not_grow_with_the_subtree() {
    let deep = nest(200);
    let shallow = nest(1);
    assert_eq!(form_name(&deep.kind), form_name(&shallow.kind));
    assert_eq!(form_name(&deep.kind), "the operator `+`");

    let deep = Expr::new(
        ExprKind::Prefix {
            op: PrefixOp::Minus,
            operand: Box::new(nest(200)),
        },
        0..1,
    );
    let shallow = Expr::new(
        ExprKind::Prefix {
            op: PrefixOp::Minus,
            operand: Box::new(literal()),
        },
        0..1,
    );
    assert_eq!(form_name(&deep.kind), form_name(&shallow.kind));
    assert_eq!(form_name(&deep.kind), "the prefix operator `-`");
}

// ---- the fragment's lifetime (I7) ----

/// Step 4's test, and the one property `INTERPRET` has that no other
/// instruction does: a name bound inside fragment text outlives the
/// fragment, so a *later, separate* fragment reads it back.
#[test]
fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
    let outcome = run_program(
        TEST_PATH,
        b"interpret \"zork = 42\"\ninterpret \"say zork\"\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(outcome.stdout, b"42\n");
}

/// Step 2, and the reason the spike exists in the shape it does.
/// ```text
/// zzz = 'from the enclosing frame'
/// interpret "say zzz"
/// interpret "zork = 42"
/// interpret "say zork"
/// zzz = zzz || '!'
/// interpret "say zzz"
/// ```
/// ```text
/// from the enclosing frame
/// 42
/// from the enclosing frame!
/// ```
#[test]
fn a_fragment_shares_the_enclosing_frames_variable_pool() {
    let program = b"zzz = 'from the enclosing frame'\n\
                    interpret \"say zzz\"\n\
                    interpret \"zork = 42\"\n\
                    interpret \"say zork\"\n\
                    zzz = zzz || '!'\n\
                    interpret \"say zzz\"\n";
    let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        outcome.stdout,
        b"from the enclosing frame\n42\nfrom the enclosing frame!\n"
    );
}

/// `EXIT` inside a fragment ends the *program*, not the fragment, so control
/// leaves the nested loop and the enclosing one together and both `Rc` locals
/// drop in order.
/// ```text
/// say 'before'
/// interpret "say 'inside'"
/// interpret "exit"
/// say 'after'
/// ```
#[test]
fn an_exit_inside_a_fragment_ends_the_program() {
    let program = b"say 'before'\n\
                    interpret \"say 'inside'\"\n\
                    interpret \"exit\"\n\
                    say 'after'\n";
    let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(outcome.stdout, b"before\ninside\n");
}

/// A condition raised inside an `INTERPRET` fragment reports
/// **both** clauses, and this is the whole report, byte for byte, at the
/// one level `run_program` can see it.
/// ```text
///      3 *-*       say 2 & 1;
///      3 *-*     interpret "do jj = 1 to 1; say 2 & 1; end"
/// Error 34 running <path> line 3:  Logical value not 0 or 1.
/// Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
/// ```
#[test]
fn a_raise_inside_a_fragment_reports_both_clauses() {
    let program = b"do kk = 1 to 1\n\
                    do mm = 1 to 1\n\
                    interpret \"do jj = 1 to 1; say 2 & 1; end\"\n\
                    end\n\
                    end\n";
    let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 222);
    assert_eq!(outcome.stdout, b"");
    assert_eq!(
        String::from_utf8(outcome.stderr).unwrap(),
        format!(
            concat!(
                "     3 *-*       say 2 & 1;\n",
                "     3 *-*     interpret \"do jj = 1 to 1; say 2 & 1; end\"\n",
                "Error 34 running {path} line 3:  Logical value not 0 or 1.\n",
                "Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"2\".\n",
            ),
            path = TEST_PATH
        )
    );
}

/// Review round 1, F1 and its neighbours: the activation base survives
/// every construct inside the fragment that writes an indent of its own.
#[test]
fn a_fragments_activation_base_survives_every_indent_writer_inside_it() {
    // (program, expected stderr with `{path}` for the program's path)
    let rows: &[(&str, &str)] = &[
        (
            "do z = 1 to 1\n\
             interpret \"select; when 1 = 0 then nop; otherwise nop; end; say 1/0\"\n\
             end\n",
            concat!(
                "     2 *-*   say 1/0\n",
                "     2 *-*   interpret \"select; when 1 = 0 then nop; otherwise nop; \
                 end; say 1/0\"\n",
                "Error 42 running {path} line 2:  Arithmetic overflow/underflow.\n",
                "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
            ),
        ),
        (
            "do z = 1 to 1\n\
             interpret \"do jj = 1 to 1; leave zz; end\"\n\
             end\n",
            concat!(
                "     2 *-*   leave zz;\n",
                "     2 *-*   interpret \"do jj = 1 to 1; leave zz; end\"\n",
                "Error 28 running {path} line 2:  Invalid LEAVE or ITERATE.\n",
                "Error 28.3:  Symbol following LEAVE (\"ZZ\") must either match the \
                 label of a current loop or block instruction.\n",
            ),
        ),
        (
            "do z = 1 to 1\n\
             select case 2\n\
             \x20 when 2 then\n\
             \x20   when 3 then nop\n\
             \x20 otherwise interpret \"do jj = 1 to 1; say 1/0; end\"\n\
             end\n\
             end\n",
            concat!(
                "     5 *-*             say 1/0;\n",
                "     5 *-*           interpret \"do jj = 1 to 1; say 1/0; end\"\n",
                "Error 42 running {path} line 5:  Arithmetic overflow/underflow.\n",
                "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
            ),
        ),
    ];
    for (index, (program, expected)) in rows.iter().enumerate() {
        let outcome = run_program(
            TEST_PATH,
            program.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            expected.replace("{path}", TEST_PATH),
            "row {index}"
        );
    }
}

/// Review round 1, F2: the `WHEN` scan's own echo carries the offsets too.
#[test]
fn a_when_scan_inside_a_fragment_echoes_at_the_fragments_own_indent() {
    let program = "trace r\n\
                   do\n\
                   interpret \"select; when 1 = 1 then nop; end; nop\"\n\
                   end\n";
    let outcome = run_program(
        TEST_PATH,
        program.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, b"");
    assert_eq!(
        String::from_utf8(outcome.stderr).unwrap(),
        concat!(
            "     2 *-* do\n",
            "     3 *-*   interpret \"select; when 1 = 1 then nop; end; nop\"\n",
            "       >>>     \"select; when 1 = 1 then nop; end; nop\"\n",
            "     3 *-*   select;\n",
            "     3 *-*     when 1 = 1 \n",
            "       >>>       \"1\"\n",
            "     3 *-*       then\n",
            "     3 *-*         nop;\n",
            "     3 *-*   nop\n",
            "     4 *-* end\n",
        )
    );
}

/// A fragment that does not parse raises the oracle's own condition
/// instead of failing loudly.
#[test]
fn a_fragment_that_does_not_parse_raises_the_oracles_condition() {
    let outcome = run_program(
        TEST_PATH,
        b"say 1\ninterpret \"do forever then\"\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 229);
    assert_eq!(outcome.stdout, b"1\n");
    assert_eq!(
        String::from_utf8(outcome.stderr).unwrap(),
        format!(
            concat!(
                "     2 *-* interpret \"do forever then\"\n",
                "Error 27 running {path} line 2:  Invalid DO or LOOP syntax.\n",
                "Error 27.901:  Incorrect data following FOREVER keyword on the loop; \
                 found \"&1\".\n",
            ),
            path = TEST_PATH
        )
    );

    let outcome = run_program(
        TEST_PATH,
        b"say 1\ninterpret \"if\"\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 221);
}

/// The reported span comes from one call chain, so what else the program
/// evaluated cannot change it.
#[test]
fn the_stack_span_does_not_depend_on_what_else_the_program_evaluated() {
    let mut alone = b"say 'a'".to_vec();
    for _ in 1..1_000 {
        alone.extend_from_slice(b"||''");
    }
    alone.push(b'\n');

    let mut then_a_fragment = alone.clone();
    then_a_fragment.extend_from_slice(b"interpret \"say 'b'\"\n");

    let on_eval = || crate::Invocation::none();
    let alone = run_program(TEST_PATH, alone, on_eval());
    let then_a_fragment = run_program(TEST_PATH, then_a_fragment, on_eval());

    assert_eq!(alone.exit_code, 0, "stderr: {:?}", alone.stderr);
    assert_eq!(
        then_a_fragment.exit_code, 0,
        "stderr: {:?}",
        then_a_fragment.stderr
    );
    assert_eq!(
        alone.stack.max_depth, then_a_fragment.stack.max_depth,
        "the fragment's own evaluation is shallow, so it must not move the maximum"
    );
    assert_eq!(
        alone.stack.bytes, then_a_fragment.stack.bytes,
        "the span must come from the chain that reached the maximum, not from the last \
         top-level evaluation to start"
    );
}

// ---- R9: ::CLASS/::METHOD/::ATTRIBUTE create and record in
// `Interp::classes`. No corpus program can see this yet -- reading a
// class object back needs a message send, and `ExprKind::Message` still
// fails loudly (Task 5's), so this is witnessed here, against the
// registry `install_directives` leaves behind, instead. ----

/// Parses `text` and installs its directives against a fresh `Interp`,
/// under `ProgramId(0)`, handing both back so a test can read the
/// registry state left behind.
fn installed(text: &[u8]) -> (Interp, Rc<Program>) {
    let program = Rc::new(parse_program(text.to_vec()).expect("test program parses"));
    let mut interp = Interp::new();
    interp
        .install_directives(ProgramId(0), &program)
        .expect("test program installs without a raised condition");
    (interp, program)
}

/// The class `::CLASS name` installed, read out of the package's own
/// table.
fn installed_class(interp: &Interp, name: &str) -> rexx_core::ObjRef {
    interp.package_classes[&ProgramId(0)][name.as_bytes()]
}

/// Had `::CLASS` not created anything, the package's own table would hold
/// nothing and the index below would panic.
#[test]
fn a_bare_class_directive_creates_a_class_object() {
    let (interp, _program) = installed(b"say 'main ran'\n::class Foo\n");
    let id = installed_class(&interp, "FOO");
    assert!(interp.heap.is_class(id), "a class identity, not a value");
}

/// **An installed class is not an environment entry**, which is the whole
/// of why `install_class` does not register the name.
#[test]
fn an_installed_class_does_not_displace_the_environments_own_entry() {
    let (mut interp, _program) = installed(b"say 'main ran'\n::class array\n");
    let installed = installed_class(&interp, "ARRAY");
    let native = interp.classes().lookup("Array").expect("Array is native");
    assert_ne!(installed, native);
    // Upcased, because the directive names the class with a symbol and
    // the scanner interns a symbol upcased -- which is why the oracle
    // prints `The ARRAY class` for it and `The Array class` for the
    // environment's own.
    assert_eq!(interp.classes().id_string(installed), "ARRAY");
    assert_eq!(interp.classes().id_string(native), "Array");
}

/// A bare `::CLASS` is `subclass Object`, `metaclass Class`, which is
/// what `RexxClass::subclass`'s own defaults give it. Had `install_class`
/// left the superclass out, or named some other class, this would
/// catch it.
#[test]
fn a_bare_class_directive_subclasses_object() {
    let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n");
    let id = installed_class(&interp, "FOO");
    let object = interp.classes().lookup("Object").unwrap();
    let class = interp.classes().lookup("Class").unwrap();
    assert_eq!(interp.classes().superclass(id), Some(object));
    assert_eq!(interp.classes().metaclass(id), class);
}

/// **`~metaClass` and `~class` are two different fields of a class
/// object, and they part iff the superclass is a metaclass and is not the
/// named-or-inherited metaclass.**
/// ```text
/// ::class S  MIXINCLASS Class         ~metaClass Class  ~class Class   same   stated, cannot fail
/// ::class M1 MIXINCLASS Class         ~metaClass Class  ~class Class   same
/// ::class T  SUBCLASS S METACLASS M1  ~metaClass S      ~class M1      part   asserted
/// ::class T2 SUBCLASS S               ~metaClass S      ~class Class   part   asserted
/// ::class K  METACLASS M1             ~metaClass M1     ~class M1      same   asserted
/// ::class P                           ~metaClass Class  ~class Class   same   asserted
/// ```
#[test]
fn a_class_objects_metaclass_and_its_class_are_separate_fields() {
    let (mut interp, _program) = installed(
        b"say 'main ran'\n\
          ::class M1 mixinclass class\n\
          ::class S mixinclass class\n\
          ::class T subclass S metaclass M1\n\
          ::class T2 subclass S\n\
          ::class K metaclass M1\n\
          ::class P\n",
    );
    let class = interp.classes().lookup("Class").unwrap();
    let m1 = installed_class(&interp, "M1");
    let s = installed_class(&interp, "S");
    let t = installed_class(&interp, "T");
    let t2 = installed_class(&interp, "T2");
    let k = installed_class(&interp, "K");
    let p = installed_class(&interp, "P");

    // Derived from a metaclass and naming one: the two fields disagree,
    // and each holds what the other does not.
    assert_eq!(interp.classes().metaclass(t), s, "T~metaClass");
    assert_eq!(interp.classes().class_of(t), m1, "T~class");
    // Derived from a metaclass, naming none: they disagree here too, so
    // the split is not an artifact of writing METACLASS down.
    assert_eq!(interp.classes().metaclass(t2), s, "T2~metaClass");
    assert_eq!(interp.classes().class_of(t2), class, "T2~class");
    // Derived from a metaclass and yet the two agree, which is the row
    // that refutes "the fields part wherever a class derives from a
    // metaclass": deriving from one is necessary and is not sufficient.
    assert_eq!(interp.classes().metaclass(s), class, "S~metaClass");
    assert_eq!(interp.classes().class_of(s), class, "S~class");
    // Naming one under a superclass that is not a metaclass, and naming
    // none at all: nothing overrides, and the two agree. Without these
    // the test would admit a build that simply answered different things.
    assert_eq!(interp.classes().metaclass(k), m1, "K~metaClass");
    assert_eq!(interp.classes().class_of(k), m1, "K~class");
    assert_eq!(interp.classes().metaclass(p), class, "P~metaClass");
    assert_eq!(interp.classes().class_of(p), class, "P~class");
}

/// The other half of the same install: a user class inherits `.Object`'s
/// own instance methods through the flattened cascade, which is what a
/// send to one of its instances would resolve against. Had
/// `install_class` recorded no superclass, this set would hold `BAR`
/// alone.
#[test]
fn a_user_class_inherits_objects_instance_methods() {
    let (mut interp, _program) =
        installed(b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n");
    let id = installed_class(&interp, "FOO");
    let names = interp.classes().instance_method_names(id);
    assert!(names.contains("BAR"));
    assert!(names.contains("HASMETHOD"));
}

/// Had `::METHOD` landed in the class dictionary instead of the instance
/// one (or nowhere), one side of this pair would be wrong.
#[test]
fn a_method_directive_lands_in_the_classs_instance_dictionary() {
    let (mut interp, _program) =
        installed(b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n");
    let id = installed_class(&interp, "FOO");
    assert!(
        interp
            .classes()
            .own_instance_method_names(id)
            .contains("BAR")
    );
    assert!(!interp.classes().own_class_method_names(id).contains("BAR"));
}

/// `::METHOD ... CLASS`'s own side of the same pair.
#[test]
fn a_class_method_directive_lands_in_the_classs_class_dictionary() {
    let (mut interp, _program) =
        installed(b"say 'main ran'\n::class Foo\n::method bar class\n  return 1\n");
    let id = installed_class(&interp, "FOO");
    assert!(interp.classes().own_class_method_names(id).contains("BAR"));
    assert!(
        !interp
            .classes()
            .own_instance_method_names(id)
            .contains("BAR")
    );
}

/// The `UNINIT` flags, **through the directive path** -- which is the
/// half `rexx-classes`' own graph-API test cannot reach.
#[test]
fn the_uninit_flags_are_set_for_the_classes_a_file_declares() {
    let (mut interp, _program) = installed(
        b"say 'main ran'
              ::class Base
              ::method uninit
  return
              ::class Kid subclass Base
              ::class Grandkid subclass Kid
              ::class Plain
              ::class Plainkid subclass Plain
",
    );
    let base = installed_class(&interp, "BASE");
    let kid = installed_class(&interp, "KID");
    let grandkid = installed_class(&interp, "GRANDKID");
    let plain = installed_class(&interp, "PLAIN");
    let plainkid = installed_class(&interp, "PLAINKID");

    // The class that declares it, and the ones that reach it through the
    // flattened behaviour the oracle's `checkUninit` reads.
    assert!(interp.classes().has_uninit(base), "the declaring class");
    assert!(interp.classes().has_uninit(kid), "its subclass");
    assert!(
        interp.classes().has_uninit(grandkid),
        "and one generation further down"
    );

    // The separate flag, propagated rather than looked up.
    assert!(!interp.classes().parent_has_uninit(base));
    assert!(interp.classes().parent_has_uninit(kid));
    assert!(interp.classes().parent_has_uninit(grandkid));

    // The negative rows: an ancestry with no UNINIT in it leaves both
    // flags clear, so a build that set them unconditionally fails here.
    assert!(!interp.classes().has_uninit(plain));
    assert!(!interp.classes().has_uninit(plainkid));
    assert!(!interp.classes().parent_has_uninit(plainkid));
}

/// A class-side `::METHOD uninit CLASS` sets neither flag, and that is
/// the oracle's answer rather than a gap.
#[test]
fn a_class_side_uninit_sets_neither_flag() {
    let (mut interp, _program) = installed(
        b"say 'main ran'
              ::class K
              ::method uninit class
  return
",
    );
    let id = installed_class(&interp, "K");
    assert!(!interp.classes().has_uninit(id));
    assert!(!interp.classes().parent_has_uninit(id));
    assert!(
        interp
            .classes()
            .own_class_method_names(id)
            .contains("UNINIT"),
        "the method did install, on the class side"
    );
}

/// Neither `GET` nor `SET`: both accessor names install. Had the `=`
/// suffix been on the wrong name, or missing, one side of this pair
/// would fail.
#[test]
fn an_attribute_with_no_style_installs_both_accessor_names() {
    let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n::attribute baz\n");
    let id = installed_class(&interp, "FOO");
    let names = interp.classes().own_instance_method_names(id);
    assert!(names.contains("BAZ"));
    assert!(names.contains("BAZ="));
}

/// `GET` alone installs only the getter -- had `install_attribute`
/// always installed both names regardless of style, `BAZ=` would be
/// present here too.
#[test]
fn an_attribute_get_installs_only_the_getter() {
    let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n::attribute baz get\n");
    let id = installed_class(&interp, "FOO");
    let names = interp.classes().own_instance_method_names(id);
    assert!(names.contains("BAZ"));
    assert!(!names.contains("BAZ="));
}

/// `::METHOD ... ATTRIBUTE` generates the same pair `::ATTRIBUTE` does,
/// and each half is recorded as the half it is.
#[test]
fn a_method_attribute_installs_a_getter_and_a_setter() {
    let (mut interp, _program) =
        installed(b"say 'main ran'\n::class Foo\n::method baz attribute\n");
    let id = installed_class(&interp, "FOO");
    let names = interp.classes().own_instance_method_names(id);
    assert!(names.contains("BAZ"));
    assert!(names.contains("BAZ="));
    assert_eq!(generated_kinds(&interp), vec!["Getter", "Setter"]);
    assert!(
        interp.method_bodies.is_empty(),
        "a generated accessor is not a row of the body table"
    );
}

/// `ABSTRACT` under `ATTRIBUTE` replaces both halves rather than one, on
/// either directive.
#[test]
fn an_abstract_accessor_pair_is_abstract_on_both_halves() {
    for source in [
        b"say 'main ran'\n::class Foo\n::method baz class abstract attribute\n".to_vec(),
        b"say 'main ran'\n::class Foo\n::attribute baz class abstract\n".to_vec(),
    ] {
        let (interp, _program) = installed(&source);
        assert_eq!(
            generated_kinds(&interp),
            vec!["Abstract", "Abstract"],
            "{:?}",
            String::from_utf8_lossy(&source)
        );
    }
}

/// A `DELEGATE` method is a row of the generated table and not of the
/// body one, and under `ATTRIBUTE` it is two rows rather than one.
#[test]
fn a_delegate_method_is_a_generated_method() {
    let (interp, _program) =
        installed(b"say 'main ran'\n::class Foo\n::method baz class delegate p\n");
    assert_eq!(generated_kinds(&interp), vec!["Delegate"]);
    assert!(
        interp.method_bodies.is_empty(),
        "a delegate method is not a row of the body table"
    );

    for source in [
        b"say 'main ran'\n::class Foo\n::method baz class delegate p attribute\n".to_vec(),
        b"say 'main ran'\n::class Foo\n::attribute baz class delegate p\n".to_vec(),
    ] {
        let (mut interp, _program) = installed(&source);
        assert_eq!(
            generated_kinds(&interp),
            vec!["Delegate", "Delegate"],
            "{:?}",
            String::from_utf8_lossy(&source)
        );
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().own_class_method_names(id);
        assert!(names.contains("BAZ"), "{names:?}");
        assert!(names.contains("BAZ="), "{names:?}");
    }
}

/// Every generated method `install_directives` recorded, as its `Debug`
/// spelling, in sorted order. A `Vec` rather than a set so a pair
/// recorded as halves of the same kind reads differently from one of
/// each.
fn generated_kinds(interp: &Interp) -> Vec<String> {
    let mut kinds: Vec<String> = interp
        .generated_methods
        .values()
        .map(|generated| format!("{:?}", generated.kind))
        .collect();
    kinds.sort();
    kinds
}

/// A loose `::METHOD` with no preceding `::CLASS` installs on the oracle
/// (measured, rc 0 "main ran") and has nothing here to attach to -- had
/// `install_directives` recorded it against a stale or default class id
/// instead of skipping it, `method_bodies` would be non-empty here.
#[test]
fn a_loose_method_with_no_preceding_class_is_not_recorded() {
    let (interp, _program) = installed(b"say 'main ran'\n::method bar\n  return 1\n");
    assert!(interp.method_bodies.is_empty());
}

/// The "bodies are stored" half of R9: `method_bodies` names the exact
/// directive a method's own body came from, keyed by the identity the
/// registry minted for it. Had `record_method_body` recorded the wrong
/// directive index, or keyed the two methods the same way, this would
/// catch it; had a mint been recorded twice, the `debug_assert!` inside
/// `record_method_body` catches that first, in every debug build
/// including the workspace's own gate.
#[test]
fn method_bodies_names_each_directive_by_its_minted_identity() {
    let (mut interp, program) = installed(
        b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n::method baz class\n  return 2\n",
    );
    let index_of = |name: &[u8]| -> usize {
        program
            .directives
            .iter()
            .position(|d| matches!(&d.kind, DirectiveKind::Method(m) if &*m.name == name))
            .expect("the ::METHOD directive is in the program")
    };
    let id = installed_class(&interp, "FOO");
    // A bare symbol's own name is already upcased by the scanner (a
    // quoted literal is the shape that would keep the source case), so
    // `bar`/`baz class` in the source above are `BAR`/`BAZ` here.
    let (_, bar) = interp
        .classes()
        .lookup_instance_method(id, "BAR")
        .expect("::method bar resolves");
    let (_, baz) = interp
        .classes()
        .lookup_class_method(id, "BAZ")
        .expect("::method baz class resolves");
    assert_eq!(interp.method_bodies.len(), 2);
    assert_eq!(interp.method_bodies[&bar].program, ProgramId(0));
    assert_eq!(interp.method_bodies[&bar].directive, index_of(b"BAR"));
    assert_eq!(interp.method_bodies[&baz].program, ProgramId(0));
    assert_eq!(interp.method_bodies[&baz].directive, index_of(b"BAZ"));
}

/// **Every class the interpreter's own library declares carries
/// `REXX_DEFINED`** -- asserted over the set the bootstrap leaves behind
/// rather than on the classes a corpus row happens to name.
#[test]
fn every_class_the_library_declares_carries_the_rexx_defined_flag() {
    let mut interp = Interp::new();
    interp
        .bootstrap_library()
        .expect("the library bootstrap runs");
    let programs = interp.library_programs.clone();
    let mut open: Vec<String> = Vec::new();
    let mut checked = 0;
    let mut non_public = 0;
    for program in programs {
        // `get`, not an index: an embedded file declaring no `::CLASS`
        // has no table at all, which `PlatformObjects.orx` is.
        let Some(table) = interp.package_classes.get(&program) else {
            continue;
        };
        let classes: Vec<(Vec<u8>, rexx_core::ObjRef)> = table
            .iter()
            .map(|(name, id)| (name.to_vec(), *id))
            .collect();
        let public = interp
            .package_public_classes
            .get(&program)
            .cloned()
            .unwrap_or_default();
        for (name, id) in classes {
            checked += 1;
            if !public.contains_key(name.as_slice()) {
                non_public += 1;
            }
            if !interp.classes().is_rexx_defined(id) {
                open.push(String::from_utf8_lossy(&name).into_owned());
            }
        }
    }
    open.sort_unstable();
    assert!(
        open.is_empty(),
        "the library declared classes a program can still mutate: {open:?}"
    );
    // Anti-vacuity, both halves. An empty table, or a bootstrap that
    // installed only public classes, would pass the loop above by
    // having nothing in it to fail.
    assert!(checked > 0, "the library declared no classes at all");
    assert!(
        non_public > 0,
        "no class without PUBLIC was seen, so the half the corpus cannot \
         reach is not what this test read"
    );
}

/// A native activation's local references are roots while its table is on
/// [`Interp::native_handles`] and nothing once it is popped, which is what
/// makes a handle outliving its activation a lookup miss (D5).
#[test]
fn a_native_activations_local_references_are_roots_only_while_it_lives() {
    let mut interp = Interp::new();
    let object = interp.heap.alloc(rexx_core::Body::Array {
        slots: Vec::new(),
        dimensions: None,
    });

    let mut frame = crate::NativeFrame {
        owner: rexx_core::ObjRef::NIL,
        scope: rexx_core::ObjRef::NIL,
        method: true,
        receiver: rexx_core::ObjRef::NIL,
        name: Vec::new(),
        arguments: Vec::new(),
        argument_list: None,
        locals: rexx_api::handles::Table::new(),
        raised: None,
        additional: None,
        result: None,
        condition: None,
        code: None,
        kept: std::collections::HashSet::new(),
    };
    let handle = frame.locals.register(object);
    interp.native_handles.push(frame);
    interp.collect_now();
    assert!(
        interp.heap.get(object).is_some(),
        "a live native activation's local reference was collected"
    );
    assert_eq!(
        interp.native_handles[0].locals.resolve(handle),
        Some(object)
    );

    interp.native_handles.pop();
    interp.collect_now();
    assert!(
        interp.heap.get(object).is_none(),
        "the table was popped and nothing else held the object, so a \
         collection that leaves it alive means these are rooted by \
         something other than the destructure this test is about"
    );
}

/// The receiver, the arguments and the argument array a native activation
/// holds for its conversions are roots while it lives, each one alone.
#[test]
fn a_native_activations_call_state_is_rooted_only_while_it_lives() {
    let mut interp = Interp::new();
    let fresh = |interp: &mut Interp| {
        interp.heap.alloc(rexx_core::Body::Array {
            slots: Vec::new(),
            dimensions: None,
        })
    };
    let (receiver, argument, list) = (fresh(&mut interp), fresh(&mut interp), fresh(&mut interp));
    let (additional, result, condition) =
        (fresh(&mut interp), fresh(&mut interp), fresh(&mut interp));
    interp.native_handles.push(crate::NativeFrame {
        owner: rexx_core::ObjRef::NIL,
        scope: rexx_core::ObjRef::NIL,
        method: true,
        receiver,
        name: b"NAME".to_vec(),
        arguments: vec![None, Some(argument)],
        argument_list: Some(list),
        locals: rexx_api::handles::Table::new(),
        raised: None,
        additional: Some(additional),
        result: Some(result),
        condition: Some(condition),
        code: None,
        kept: std::collections::HashSet::new(),
    });
    let held = [
        (receiver, "receiver"),
        (argument, "argument"),
        (list, "list"),
        (additional, "held ADDITIONAL"),
        (result, "held RESULT"),
        (condition, "held condition object"),
    ];
    interp.collect_now();
    for (held, what) in held {
        assert!(interp.heap.get(held).is_some(), "the {what} was collected");
    }
    interp.native_handles.pop();
    interp.collect_now();
    for (held, what) in held {
        assert!(
            interp.heap.get(held).is_none(),
            "the {what} outlived its frame, so something else roots it"
        );
    }
}
