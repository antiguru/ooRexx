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

use super::*;

/// The caller a resolution asked for outside any send is asking as:
/// neither a receiver nor a package, which is the state
/// `RexxObject::checkPrivate` and `RexxObject::checkPackage` each refuse
/// (`classes/ObjectClass.cpp:622`-`:626`, `:665`-`:669`).
fn no_caller() -> Caller {
    Caller {
        receiver: None,
        package: CallerPackage::NoActivation,
    }
}

/// The object model is built on first use and not in `Interp::new`.
#[test]
fn the_object_model_is_built_on_first_use() {
    let mut interp = Interp::new();
    assert!(interp.object_model.is_none());
    let _ = interp.classes();
    assert!(interp.object_model.is_some());
}

/// A program that neither declares a class nor sends a message never
/// touches the object model.
#[test]
fn a_program_with_no_send_and_no_class_never_builds_the_object_model() {
    let mut interp = Interp::new();
    let program = rexx_parse::parse_program(b"say 1 + 1\n".to_vec()).expect("it parses");
    interp
        .install_directives(crate::ProgramId(0), &std::rc::Rc::new(program))
        .expect("no directives to install");
    assert!(interp.object_model.is_none());
}

/// An ordinary resolution answers the class the definition came from,
/// which is not always the receiver's own class: `LENGTH` is `String`'s
/// and `ISNIL` is `Object`'s, and one string receiver reaches both.
#[test]
fn an_ordinary_resolution_answers_the_defining_scope() {
    let mut interp = Interp::new();
    let receiver = interp.text(b"abc");
    let string = interp.classes().lookup("String").expect("String is native");
    let object = interp.classes().lookup("Object").expect("Object is native");

    let own = interp
        .resolve(receiver, b"LENGTH", None, no_caller())
        .expect("String answers LENGTH");
    assert_eq!(own.scope, string);
    let inherited = interp
        .resolve(receiver, b"ISNIL", None, no_caller())
        .expect("Object's ISNIL reaches a String receiver");
    assert_eq!(inherited.scope, object);
    assert_eq!(
        interp
            .resolve(receiver, b"NOSUCHMETHOD", None, no_caller())
            .err(),
        Some(Miss::NoMethod)
    );
}

/// **The start-scope argument, at the seam rather than through a
/// program.**
#[test]
fn a_start_scope_hides_a_method_defined_after_it_and_not_one_defined_before() {
    let mut interp = Interp::new();
    let receiver = interp.text(b"abc");
    let string = interp.classes().lookup("String").expect("String is native");
    let object = interp.classes().lookup("Object").expect("Object is native");

    assert!(
        interp
            .resolve(receiver, b"ISNIL", Some(object), no_caller())
            .is_ok()
    );
    assert!(
        interp
            .resolve(receiver, b"LENGTH", Some(object), no_caller())
            .is_err()
    );
    assert!(
        interp
            .resolve(receiver, b"ISNIL", Some(string), no_caller())
            .is_ok()
    );
    assert!(
        interp
            .resolve(receiver, b"LENGTH", Some(string), no_caller())
            .is_ok()
    );
}

/// A name the class answers with no implementation here is **loud**, and
/// a name it does not answer is the oracle's own 97.1. The two are
/// different answers and the pair is what keeps them apart: a build that
/// raised for both would let a program expecting a working `String`
/// method pass against a gap.
#[test]
fn an_unimplemented_method_is_loud_where_an_unknown_one_is_a_condition() {
    let mut interp = Interp::new();
    let receiver = interp.text(b"abc");
    let string = interp.classes().lookup("String").expect("String is native");
    let answered = interp.classes().instance_method_names(string);
    // **The candidate is found in the registry and only then sent.**
    // Sending every name until one refuses used to do it, and that stopped
    // working the moment a numeric method landed early in the order: a
    // body that reads `NUMERIC DIGITS` wants a live activation, and this
    // test has none, so the search panicked before it reached a loud name.
    let rows: Vec<&str> = NATIVE_METHODS
        .iter()
        .chain(string::NATIVE_METHODS)
        .chain(hash::NATIVE_METHODS)
        .chain(collection::list::NATIVE_METHODS)
        .chain(collection::queue::NATIVE_METHODS)
        .chain(array::sort::NATIVE_METHODS)
        .chain(array::surface::NATIVE_METHODS)
        .chain(collection::supplier::NATIVE_METHODS)
        .filter(|(class, ..)| *class == "String")
        .map(|(_, method, ..)| *method)
        .collect();
    let mut unimplemented = None;
    for name in &answered {
        if rows.contains(&name.as_str()) {
            continue;
        }
        if matches!(
            interp.send_message(receiver, name.as_bytes(), None, &[], no_caller()),
            Err(Failure::Loud(_))
        ) {
            unimplemented = Some(name.clone());
            break;
        }
    }
    assert!(
        unimplemented.is_some(),
        "every name .String's behaviour answers now has an implementation, so this test \
         has no subject left and its pair has to be rebuilt on something else"
    );
    assert!(matches!(
        interp.send_message(receiver, b"NOSUCHMETHOD", None, &[], no_caller()),
        Err(Failure::Raised(_))
    ));
}

/// **A class identity used as a receiver resolves against that class's
/// own class behaviour, never through the arena.**
#[test]
fn a_class_identity_used_as_a_receiver_answers_from_the_class_side() {
    let mut interp = Interp::new();
    // Each is longer than a handle can hold, so each really takes a slot
    // instead of travelling inline.
    let mut last = ObjRef::NIL;
    for text in [
        &b"first-occupant!"[..],
        b"second-occupant!",
        b"third-occupant!!!",
    ] {
        last = interp.text(text);
        assert!(
            interp.heap.get(last).is_some(),
            "the occupant went inline instead of into a slot"
        );
    }
    let Decoded::Heap { slot, .. } = last.decode() else {
        panic!("an arena handle")
    };
    assert_eq!(
        slot, 2,
        "this test rests on the third occupant holding the slot .String's identity would \
         have been equal to; the arena's allocation order moved"
    );

    let class = interp.classes().lookup("String").expect("String is native");
    assert!(matches!(
        interp.send_message(class, b"LENGTH", None, &[], no_caller()),
        Err(Failure::Raised(_))
    ));
    let name = interp.text(b"LENGTH");
    assert_eq!(
        interp
            .send_message(class, b"HASMETHOD", None, &[Some(name)], no_caller())
            .expect("Object's HASMETHOD reaches a class object"),
        Some(interp.counted(0)),
        "a class object was asked about its instances' behaviour instead of its own"
    );
}

/// A `.Pointer` over `address`, minted the way the API's `NewPointer`
/// does: nothing in a Rexx program can build one, which is why this
/// stands where a source program would.
fn pointer(interp: &mut Interp, address: usize) -> ObjRef {
    let class = interp
        .classes()
        .lookup("Pointer")
        .expect("Pointer is a native class");
    let behaviour = interp.classes().instance_behaviour_handle(class);
    let body = Body::pointer(class, behaviour, std::ptr::without_provenance_mut(address));
    let object = interp.alloc_with(rexx_core::BehaviourId::OBJECT, body);
    interp.roots.push_temp(object);
    object
}

/// A `.Pointer` answers its address, its class and its own name, all
/// measured 2026-09-14 against the oracle through a `CSELF` an
/// `expose`-ing method handed back.
#[test]
fn a_pointer_renders_as_its_address_and_names_its_class() {
    let mut interp = Interp::new();
    let object = pointer(&mut interp, 0x55f0_6283_2410);
    assert_eq!(interp.to_text(object).into_owned(), b"0x55f062832410");
    assert_eq!(interp.text_len(object), 14);

    let class = interp
        .send_message(object, b"CLASS", None, &[], no_caller())
        .expect("a Pointer answers CLASS")
        .expect("a class");
    let id = interp
        .send_message(class, b"ID", None, &[], no_caller())
        .expect("a class answers ID")
        .expect("an id");
    assert_eq!(interp.to_text(id).into_owned(), b"Pointer");
    let named = interp
        .send_message(object, b"OBJECTNAME", None, &[], no_caller())
        .expect("a Pointer answers OBJECTNAME")
        .expect("a name");
    assert_eq!(interp.to_text(named).into_owned(), b"a Pointer");
    let rendered = interp
        .send_message(object, b"STRING", None, &[], no_caller())
        .expect("a Pointer answers STRING")
        .expect("a string");
    assert_eq!(interp.to_text(rendered).into_owned(), b"0x55f062832410");

    // `~objectName=` renames the object without touching its string
    // value, which is the arm a buffer is in too.
    let renamed = interp.text(b"a name of its own");
    interp
        .send_message(object, b"OBJECTNAME=", None, &[Some(renamed)], no_caller())
        .expect("a Pointer answers OBJECTNAME=");
    assert_eq!(interp.to_text(object).into_owned(), b"0x55f062832410");
}

/// A null address renders as `0x0` and answers `~isNull`, where any other
/// address does not (`interpreter/runtime/Numerics.cpp:882`,
/// `classes/PointerClass.cpp:163`).
#[test]
fn a_null_pointer_renders_as_zero_and_answers_is_null() {
    let mut interp = Interp::new();
    let null = pointer(&mut interp, 0);
    let held = pointer(&mut interp, 0x1234);
    assert_eq!(interp.to_text(null).into_owned(), b"0x0");
    for (object, expected) in [(null, 1usize), (held, 0)] {
        let answer = interp
            .send_message(object, b"ISNULL", None, &[], no_caller())
            .expect("a Pointer answers ISNULL");
        assert_eq!(answer, Some(interp.counted(expected)));
    }
}

/// Every comparison spelling: two `.Pointer`s compare on their addresses
/// and anything else compares unequal.
#[test]
fn a_pointer_compares_on_its_address_and_nothing_else() {
    let mut interp = Interp::new();
    let object = pointer(&mut interp, 0x1234);
    let same = pointer(&mut interp, 0x1234);
    let other = pointer(&mut interp, 0x5678);
    let string = interp.text(b"abc");
    let yes = interp.counted(1);
    let no = interp.counted(0);

    for name in [b"=".as_slice(), b"=="] {
        for (against, expected) in [(object, yes), (same, yes), (other, no), (string, no)] {
            let answer = interp
                .send_message(object, name, None, &[Some(against)], no_caller())
                .expect("a Pointer answers its comparisons");
            assert_eq!(answer, Some(expected), "{}", String::from_utf8_lossy(name));
        }
    }
    for name in [b"\\=".as_slice(), b"\\=="] {
        for (against, expected) in [(object, no), (same, no), (other, yes), (string, yes)] {
            let answer = interp
                .send_message(object, name, None, &[Some(against)], no_caller())
                .expect("a Pointer answers its comparisons");
            assert_eq!(answer, Some(expected), "{}", String::from_utf8_lossy(name));
        }
    }
}

/// A comparison with no argument is 93.903, measured against the oracle
/// as `p~'=='()`.
#[test]
fn a_pointer_comparison_needs_its_argument() {
    let mut interp = Interp::new();
    let object = pointer(&mut interp, 0x1234);
    let outcome = interp.send_message(object, b"==", None, &[], no_caller());
    let Err(Failure::Raised(raised)) = outcome else {
        panic!("a comparison with no argument answered instead of raising");
    };
    assert_eq!((raised.number, raised.sub), (93, 903));
}

/// Runs `source` and hands back `(exit code, stdout, stderr)`.
fn run_source(source: &str) -> (i32, String, String) {
    let outcome = crate::run_program(
        "/t.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    (
        outcome.exit_code,
        String::from_utf8_lossy(&outcome.stdout).into_owned(),
        String::from_utf8_lossy(&outcome.stderr).into_owned(),
    )
}

/// **D24's `SmallInt` behaviour arm is taken for a small integer
/// receiver**, where the general path is what a receiver whose bytes are
/// in the arena takes.
#[test]
fn a_small_integer_receiver_takes_the_small_int_arm() {
    let mut interp = Interp::new();
    let integer = ObjRef::small_int(12345).expect("12345 fits the tag");
    assert!(
        matches!(integer.decode(), Decoded::SmallInt(_)),
        "the value model stopped holding 12345 in the tag, so this test's subject is gone"
    );
    let text = interp.text(b"12345");
    assert!(
        matches!(text.decode(), Decoded::Text(_)),
        "the same digits as bytes went somewhere other than the handle's text arm"
    );

    assert_eq!(
        interp
            .receiver_kind(integer)
            .expect("a small integer answers"),
        Primitive::SmallInt,
        "a tagged integer receiver went down the general path"
    );
    assert_eq!(
        interp.receiver_kind(text).expect("a text handle answers"),
        Primitive::String
    );

    let string = interp.classes().lookup("String").expect("String is native");
    let string_behaviour = Behaviour::Instance {
        methods: interp.classes().instance_behaviour_handle(string),
        owner: string,
    };
    assert_eq!(
        interp
            .receiver_behaviour(integer)
            .expect("a small integer resolves"),
        string_behaviour,
        "the arm sends a small integer's messages somewhere other than String"
    );
    assert_eq!(
        interp
            .receiver_behaviour(text)
            .expect("a text handle resolves"),
        string_behaviour
    );

    assert_eq!(
        run_source("say 12345~length\n"),
        (0, "5\n".to_string(), String::new())
    );
}

/// **D24's receiver in the calling convention**: `SELF` is the object the
/// send was addressed to, taken from `crate::CallContext`, and not the
/// scope the resolution found the method at.
#[test]
fn a_method_send_binds_self_from_the_receiver_in_the_calling_convention() {
    assert_eq!(
        run_source(
            "say .J~m\n\
             ::class K\n\
             ::method m class\n  return self\n\
             ::class J subclass K\n"
        ),
        (0, "The J class\n".to_string(), String::new()),
        "SELF is not the receiver of the send"
    );

    let interp = Interp::new();
    assert!(
        interp.caller().receiver().is_none(),
        "a frame that is not a method's carries a receiver"
    );
}

/// **`SELF` and `SUPER` are bound before the body's first instruction**,
/// to the receiver and to the scope after the method's own.
#[test]
fn self_and_super_are_bound_before_the_bodys_first_instruction() {
    assert_eq!(
        run_source(
            "say .K~m\n\
             ::class K\n\
             ::method m class\n  return self '/' super\n"
        ),
        (
            0,
            "The K class / The Class class\n".to_string(),
            String::new()
        )
    );
    assert_eq!(
        run_source(
            "say .K~m\n\
             ::class K\n\
             ::method m class\n  interpret \"zz = self '/' super\"\n  return zz\n"
        ),
        (
            0,
            "The K class / The Class class\n".to_string(),
            String::new()
        )
    );
}

/// A native method's argument that has no string value raises 88.909,
/// and the neighbouring arguments that have one still answer.
#[test]
fn an_argument_with_no_string_value_raises_where_one_with_a_string_value_answers() {
    for source in [
        "say 'abc'~hasMethod(.String)\n",
        "say 'abc'~hasMethod(.environment)\n",
        "say 'abc'~hasMethod(.nil)\n",
        "a. = .array\nsay 'abc'~hasMethod(a.)\n",
        "a. = .nil\nsay 'abc'~hasMethod(a.)\n",
        "say .K~hasMethod(.String)\n::class K\n",
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (168, ""), "{source:?}");
        assert!(
            stderr.contains("Error 88.909:  Argument 1 must have a string value."),
            "{source:?} raised {stderr:?}"
        );
        assert!(
            stderr.contains("Compiled method \"HASMETHOD\" with scope \"Object\"."),
            "{source:?} lost the method's own traceback line: {stderr:?}"
        );
    }

    for (source, expected) in [
        ("say 'abc'~hasMethod('LENGTH')\n", "1\n"),
        ("say 'abc'~hasMethod(5)\n", "0\n"),
        ("a. = 'LENGTH'\nsay 'abc'~hasMethod(a.)\n", "1\n"),
        ("say 'abc'~hasMethod(d.)\n", "0\n"),
    ] {
        assert_eq!(
            run_source(source),
            (0, expected.to_string(), String::new()),
            "{source:?}"
        );
    }
}

/// The `::METHOD` and `::ATTRIBUTE` shapes whose body this crate cannot
/// run are **loud**, and the neighbouring shapes that it can run are not.
#[test]
fn a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run() {
    // (source, the refusal's own text after `rexx-exec: `). The tail
    // differs by row and is not a shared suffix: `Loud::method_body`
    // names Phase 5 as the owner and `Loud::accessor_variable` names
    // none, on the reasoning that constructor's doc gives.
    let refused: &[(&str, &str)] = &[
        // A generated accessor over a variable that is not a simple name.
        // Oracle rc 0 both: the stem answers `5` for the round trip and
        // the compound answers its own derived name `a.b`.
        (
            ".K~'A.' = 5\nsay .K~'A.'\n::class K\n::attribute \"a.\" class\n",
            "a generated accessor for the attribute \"a.\" is not implemented",
        ),
        (
            "say .K~'A.B'\n::class K\n::attribute \"a.b\" class\n",
            "a generated accessor for the attribute \"a.b\" is not implemented",
        ),
        // oracle rc 0, printing `4`: `USE LOCAL` binds its list against
        // the method's own scope pool.
        (
            "say .K~m\n::class K\n::method m class\n  use local zz\n  zz = 4\n  return zz\n",
            "USE LOCAL in a ::METHOD body is not implemented (Phase 5)",
        ),
    ];
    for (source, refusal) in refused {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (
                crate::NOT_IMPLEMENTED_EXIT,
                "",
                format!("rexx-exec: {refusal}\n").as_str()
            ),
            "{source:?}"
        );
    }

    for (source, expected) in [
        (
            "say .K~m\n::class K\n::method m class protected\n  return 7\n",
            "7\n",
        ),
        (
            "say .K~m\n::class K\n::method m class unguarded\n  return 7\n",
            "7\n",
        ),
        (
            "say .K~m\n::class K\n::method m class package\n  return 7\n",
            "7\n",
        ),
        (
            "say .K~a\n::class K\n::attribute a class get\n  return 11\n",
            "11\n",
        ),
    ] {
        assert_eq!(
            run_source(source),
            (0, expected.to_string(), String::new()),
            "{source:?}"
        );
    }
}

/// **A `DELEGATE` directive installs a forwarding method under every key
/// it claims**, including the setter half of the pair a `ATTRIBUTE`
/// modifier adds, and every row's bytes are the oracle's own.
#[test]
fn a_delegate_directive_forwards_under_every_key_it_claims() {
    for (source, message) in [
        ("say .K~m\n::class K\n::method m class delegate p\n", "M"),
        ("say .K~a\n::class K\n::attribute a class delegate p\n", "A"),
        (
            "say .K~a\n::class K\n::method a class delegate p attribute\n\
             ::attribute p class\n",
            "A",
        ),
        (
            ".K~a = 5\nsay 'stored'\n::class K\n\
             ::method a class delegate p attribute\n::attribute p class\n",
            "A=",
        ),
    ] {
        let clause = source.lines().next().expect("a first clause");
        let expected = format!(
            "     1 *-* {clause}\n\
             Error 97 running /t.rex line 1:  Object method not found.\n\
             Error 97.1:  Object \"P\" does not understand message \"{message}\".\n"
        );
        assert_eq!(
            run_source(source),
            (159, String::new(), expected),
            "{source:?}"
        );
    }
}

/// **A generated accessor pair reads and writes one variable in the
/// declaring scope's pool on the receiver**, and every row is measured on
/// the oracle.
#[test]
fn a_generated_accessor_pair_reads_and_writes_the_declaring_scopes_pool() {
    for (source, expected) in [
        // The round trip, through each directive.
        (
            ".K~a = 5\nsay .K~a\n::class K\n::attribute a class\n",
            "5\n",
        ),
        (
            ".K~a = 5\nsay .K~a\n::class K\n::method a class attribute\n",
            "5\n",
        ),
        // An unassigned variable reads as its derived name, which is the
        // directive's name **as written** and not the accessor's upcased
        // dictionary key: `getRetriever(name)` against
        // `addMethod(internalname)`. The quoted row is what tells the two
        // apart -- an accessor keyed on the message name would answer
        // `AB` here.
        ("say .K~a\n::class K\n::attribute a class\n", "A\n"),
        ("say .K~ab\n::class K\n::attribute \"aB\" class\n", "aB\n"),
        // And it is the same pool entry `EXPOSE` reaches, in both
        // directions.
        (
            ".K~a = 'through the setter'\nsay .K~read\n::class K\n::attribute a class\n\
             ::method read class\n  expose a\n  return a\n",
            "through the setter\n",
        ),
        (
            "x = .K~write\nsay .K~a\n::class K\n::attribute a class\n\
             ::method write class\n  expose a\n  a = 'through EXPOSE'\n  return 1\n",
            "through EXPOSE\n",
        ),
        // **Keyed on the declaring scope and on the receiver**, which is
        // one property from both sides. `.J~a` and `.K~a` name different
        // receivers and so different pools: measured, the second answers
        // its derived name after the first was assigned.
        (
            ".J~a = 5\nsay .J~a .K~a\n::class K\n::attribute a class\n\
             ::class J subclass K\n",
            "5 A\n",
        ),
        // A value is stored, not a rendering of one: the receiver goes in
        // and comes back out.
        (
            ".K~a = .K\nsay .K~a\n::class K\n::attribute a class\n",
            "The K class\n",
        ),
        // `GET` and `SET` each generate one half, and the generated half
        // reads the same pool a generated pair does.
        ("say .K~a\n::class K\n::attribute a class get\n", "A\n"),
        // A trailing omission leaves the getter's own bound satisfied.
        ("say .K~a(,)\n::class K\n::attribute a class\n", "A\n"),
        (
            ".K~a = 5\nsay 'stored'\n::class K\n::attribute a class set\n",
            "stored\n",
        ),
    ] {
        assert_eq!(
            run_source(source),
            (0, expected.to_string(), String::new()),
            "{source:?}"
        );
    }

    // **Neither `GET` nor `SET` installs the other half**, so the
    // message the directive did not generate is a name miss. Measured at
    // rc 159, one program each.
    for (source, missing) in [
        (
            ".K~a = 5\nsay 'x'\n::class K\n::attribute a class get\n",
            "A=",
        ),
        ("say .K~a\n::class K\n::attribute a class set\n", "A"),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (159, ""), "{source:?}");
        assert!(
            stderr.contains(&format!(
                "Error 97.1:  Object \"The K class\" does not understand message \
                 \"{missing}\"."
            )),
            "{source:?} reported {stderr:?}"
        );
    }

    // The setter answers nothing, which a program can read: measured,
    // `91.999` at rc 165 rather than a value the assignment produced.
    let (code, stdout, stderr) =
        run_source("r = .K~'A='(9)\nsay r\n::class K\n::attribute a class\n");
    assert_eq!((code, stdout.as_str()), (165, ""));
    assert!(
        stderr.contains("Error 91.999:  Message \"A=\" did not return a result."),
        "the setter answered a value: {stderr:?}"
    );

    // The argument bounds, which are the accessor pair's own and are the
    // C++'s (`execution/CPPCode.cpp:284`, `:334`, `:339`). Each row is
    // measured at rc 163.
    for (source, catalogue) in [
        (
            "say .K~a(1)\n::class K\n::attribute a class\n",
            "Error 93.902:  Too many arguments in invocation of method; 0 expected.",
        ),
        (
            "say .K~'A='(1,2)\n::class K\n::attribute a class\n",
            "Error 93.902:  Too many arguments in invocation of method; 1 expected.",
        ),
        (
            "say .K~'A='()\n::class K\n::attribute a class\n",
            "Error 93.903:  Missing argument in method; argument 1 is required.",
        ),
        // **A trailing omission is not an argument that arrived, and a
        // leading one is.** Measured, both at rc 163: `(,)` reaches the
        // setter as no arguments at all, and `(,5)` as two. The getter's
        // side of the same rule is the `(,)` row below, which answers.
        (
            "say .K~'A='(,)\n::class K\n::attribute a class\n",
            "Error 93.903:  Missing argument in method; argument 1 is required.",
        ),
        (
            "say .K~'A='(,5)\n::class K\n::attribute a class\n",
            "Error 93.902:  Too many arguments in invocation of method; 1 expected.",
        ),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (163, ""), "{source:?}");
        assert!(stderr.contains(catalogue), "{source:?} reported {stderr:?}");
        // **No frame of its own**, which is what separates an accessor
        // from a `NativeMethod` taking the same refusal: measured,
        // `'abc'~length(1)` carries a `Compiled method "LENGTH"` line
        // above the sending clause and none of these do.
        assert!(
            !stderr.contains("Compiled method"),
            "{source:?} grew a traceback frame the oracle does not write: {stderr:?}"
        );
    }
}

/// **An `ABSTRACT` method installs and is refused when it is sent**,
/// naming the message rather than the directive.
#[test]
fn an_abstract_send_is_refused_at_the_send_naming_the_message() {
    for (source, named) in [
        ("say .K~m\n::class K\n::method m class abstract\n", "M"),
        (
            "say .K~\"MiXeD\"\n::class K\n::method \"MiXeD\" class abstract\n",
            "MIXED",
        ),
        (
            "say .K~m\n::class K\n::method m class abstract attribute\n",
            "M",
        ),
        ("say .K~a\n::class K\n::attribute a class abstract\n", "A"),
        (".K~a = 3\n::class K\n::attribute a class abstract\n", "A="),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (163, ""), "{source:?}");
        assert!(
            stderr.contains(&format!(
                "Error 93.965:  Method {named} is ABSTRACT and cannot be directly invoked."
            )),
            "{source:?} reported {stderr:?}"
        );
        assert!(
            !stderr.contains("Compiled method"),
            "{source:?} grew a traceback frame the oracle does not write: {stderr:?}"
        );
    }

    // **Installing one is not sending one**: the directive is rc 0 with
    // the program's own output, which is what makes the row above a send
    // refusal rather than an install refusal. Measured on the oracle.
    assert_eq!(
        run_source("say 'installed'\n::class K\n::method m class abstract\n"),
        (0, "installed\n".to_string(), String::new())
    );
}

/// **A private send is refused by who is sending, and the refusal names
/// the scope that refused it.**
#[test]
fn private_sends_are_refused_by_who_is_sending() {
    let class = "\n::CLASS K\n::METHOD m CLASS PRIVATE\n  return 'inner'\n";
    for source in [
        // The program's own frame, which has no receiver.
        &format!("say .K~m{class}"),
        // A routine's frame, which has none either.
        &format!("say r(){class}::ROUTINE r\n  return .K~m\n"),
        // A sibling class in the same package.
        &format!("say .S~poke{class}::CLASS S\n::METHOD poke CLASS\n  return .K~m\n"),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (159, ""), "{source:?}");
        assert!(
            stderr.contains(
                "Error 97.2:  Object \"The K class\" cannot accept private message \
                 \"M\" from this context."
            ),
            "a refused private send must report 97.2 naming the receiver and the \
             message, got {stderr:?} for {source:?}"
        );
    }
    for source in [
        // The declaring class's own class method, sending to itself.
        &format!("say .K~outer{class}::METHOD outer CLASS\n  return self~m\n"),
        // A subclass's own class method, sending to itself: the same
        // object, and the limb a rule written over the defining scope
        // alone would refuse.
        "say .Sub~poke\n::CLASS Base\n::METHOD m CLASS PRIVATE\n  return 'inner'\n\
         ::CLASS Sub SUBCLASS Base\n::METHOD poke CLASS\n  return self~m\n",
        // A class object whose hierarchy contains the declaring scope,
        // sending to a different object.
        "say .Sub~poke\n::CLASS Base\n::METHOD m CLASS PRIVATE\n  return 'inner'\n\
         ::CLASS Sub SUBCLASS Base\n::METHOD poke CLASS\n  return .Base~m\n",
    ] {
        assert_eq!(
            run_source(source),
            (0, "inner\n".to_string(), String::new()),
            "{source:?}"
        );
    }
}

/// **The refusal is not raised at the send**: it drops the method and
/// enters the receiver's own `UNKNOWN`, and the `UNKNOWN` lookup is not
/// itself access-checked.
#[test]
fn a_refused_send_reaches_unknown_and_the_unknown_lookup_is_not_checked() {
    assert_eq!(
        run_source(
            "say .K~m\n::CLASS K\n::METHOD m CLASS PRIVATE\n  return 'never'\n\
             ::METHOD unknown CLASS\n  use arg name\n  return 'unknown saw' name\n"
        ),
        (0, "unknown saw M\n".to_string(), String::new())
    );
    assert_eq!(
        run_source(
            "say .K~zork\n::CLASS K\n::METHOD unknown CLASS PRIVATE\n\
             \x20 use arg name\n  return 'private unknown saw' name\n"
        ),
        (0, "private unknown saw ZORK\n".to_string(), String::new())
    );
}

/// **`PACKAGE`'s refusing arm**, called directly with the two callers the
/// oracle refuses: one with no activation at all, which no program
/// reaches, and one whose package is not the method's, which a
/// `::REQUIRES` of a file declaring the method does --
/// `corpus/refusal-sites.tsv`'s `package_scope_method` row names that
/// program. The allowing arm has a corpus program
/// (`corpus/lang/method_access_package_and_protected.rex`) and is
/// asserted here too, so a check that refused everything fails rather
/// than passing both refusals.
#[test]
fn package_scope_refuses_a_caller_from_another_package() {
    let method_package = Package::Program(crate::ProgramId(0));
    assert_eq!(
        Interp::check_package(method_package, no_caller()),
        Err(Miss::PackageScope),
        "a caller with no activation is `checkPackage`'s first refusal"
    );
    assert_eq!(
        Interp::check_package(
            method_package,
            Caller {
                receiver: None,
                package: CallerPackage::Package(Package::Program(crate::ProgramId(1))),
            },
        ),
        Err(Miss::PackageScope),
        "a caller in another program's package must be refused"
    );
    assert_eq!(
        Interp::check_package(
            method_package,
            Caller {
                receiver: None,
                package: CallerPackage::Package(method_package),
            },
        ),
        Ok(()),
        "the same package must be allowed"
    );
    assert_eq!(
        Interp::check_package(
            Package::Rexx,
            Caller {
                receiver: None,
                package: CallerPackage::Package(Package::Program(crate::ProgramId(0))),
            },
        ),
        Err(Miss::PackageScope),
        "the interpreter's own package is not a program's"
    );
}

/// **`checkPrivate`'s `isInstanceOf` limb, which no program in this phase
/// can reach either.**
#[test]
fn a_private_send_from_another_instance_of_the_declaring_class_is_allowed() {
    let mut interp = Interp::new();
    let sender = interp.text(b"abc");
    let receiver = interp.classes().lookup("Array").expect("Array is native");
    let string = interp.string_class();
    let caller = Caller {
        receiver: Some(sender),
        package: CallerPackage::NoActivation,
    };
    // `MethodId` is not read by the check at all -- the scope is -- so
    // the resolution below names a method that exists for the receiver
    // and nothing rests on which one it is.
    let resolution = interp
        .resolve(sender, b"LENGTH", None, no_caller())
        .expect("String answers LENGTH");
    assert_ne!(sender, receiver, "the two must not be the same object");
    assert_eq!(
        interp.check_private(
            Resolution {
                scope: string,
                method: resolution.method
            },
            receiver,
            caller,
        ),
        Ok(()),
        "a sender whose class is the declaring scope must be allowed"
    );
    let object = interp.classes().lookup("Array").expect("Array is native");
    assert_eq!(
        interp.check_private(
            Resolution {
                scope: object,
                method: resolution.method
            },
            receiver,
            caller,
        ),
        Err(Miss::Private),
        "a sender whose class is not compatible with the declaring scope must be refused"
    );
}

/// A `target~name:scope` override answers **when the scope is a class
/// object the receiver's behaviour holds**, is 88.914 when the scope is
/// not a class object at all, and is 93.957 when it is a class the
/// receiver's behaviour was never given.
#[test]
fn a_scope_override_answers_and_has_one_refusal_for_each_bad_scope() {
    assert_eq!(
        run_source("say 'abc'~length:.String\n"),
        (0, "3\n".to_string(), String::new())
    );
    let (code, stdout, stderr) = run_source("say 'abc'~length:super\n");
    assert_eq!((code, stdout.as_str()), (168, ""));
    assert!(
        stderr.contains("Error 88.914:"),
        "a non-class scope must keep the oracle's own condition, got {stderr:?}"
    );
    let (code, stdout, stderr) = run_source("say 'abc'~length:.Array\n");
    assert_eq!((code, stdout.as_str()), (163, ""));
    assert!(
        stderr.contains(
            "Error 93.957:  Target object \"abc\" is not a subclass of the message \
             override scope (The Array class)."
        ),
        "a class the receiver's behaviour does not hold must be 93.957, got {stderr:?}"
    );
}

/// A `Stem` answers a message it has no method for by forwarding it to its
/// VALUE -- `StemClass::unknownRexx`.
#[test]
fn a_stem_forwards_a_message_it_has_no_method_for_to_its_value() {
    let mut interp = Interp::new();
    let stem = interp.alloc_with(
        rexx_core::BehaviourId::STEM,
        Body::Stem {
            name: b"A."[..].into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        },
    );
    let answered = interp
        .send_message(stem, b"LENGTH", None, &[], no_caller())
        .expect("a stem forwards an unknown message to its value")
        .expect("LENGTH answers a value");
    assert_eq!(interp.to_text(answered).as_ref(), b"2");
}

/// `~identityHash` answers, and **the corpus cannot witness it**: the
/// oracle's answer is derived from the object's address, so no differential
/// row can compare the two and this test is the whole instrument.
#[test]
fn the_string_hash_matches_the_oracle_including_above_7f() {
    // Measured with `c2x(<x>~hashCode)`, read back as little-endian.
    assert_eq!(string_hash(b""), 0x0000_0000_0000_0000);
    assert_eq!(string_hash(b"a"), 0x0000_0000_0000_0061);
    assert_eq!(string_hash(b"abc"), 0x0000_0000_0001_7862);
    assert_eq!(string_hash(b"5"), 0x0000_0000_0000_0035);
    assert_eq!(string_hash(b"String"), 0x0000_0000_943a_4c31);
    // Wraps the 64-bit register rather than saturating.
    assert_eq!(
        string_hash(b"abcdefghijklmnopqrstuvwxyz0123456789"),
        0xa09f_2fd2_5824_3772
    );
    // Signed: one byte of 0xff contributes -1, not 255.
    assert_eq!(string_hash(b"\xff"), 0xffff_ffff_ffff_ffff);
    assert_eq!(string_hash(b"\x80"), 0xffff_ffff_ffff_ff80);
    assert_eq!(string_hash(b"\x7f"), 0x0000_0000_0000_007f);
}

/// `==` and not `=`, and `numeric digits 20` rather than the default:
/// measured, the answer is wider than nine significant digits, so a
/// numeric comparison at the default `DIGITS` rounds two different
/// handles' answers together and reports them equal.
#[test]
fn identity_hash_answers_a_number_that_follows_the_handle() {
    assert_eq!(
        run_source(
            "numeric digits 20\n\
             say (.Array~identityHash == .Array~identityHash)\n\
             say (.Array~identityHash == .String~identityHash)\n\
             say datatype(.Array~identityHash, 'W')\n"
        ),
        (0, "1\n0\n1\n".to_string(), String::new())
    );
}

/// Every receiver kind answers `~identityHash`, and each line is the
/// oracle's own answer, measured 2026-09-03 on three descriptors.
#[test]
fn identity_hash_answers_every_receiver_kind_as_the_oracle_does() {
    assert_eq!(
        run_source(
            "numeric digits 20\n\
             s. = 1\n\
             o = s.\n\
             say datatype(o~identityHash, 'W') datatype('abc'~identityHash, 'W')\n\
             say datatype(.Object~new~identityHash, 'W') \
                 datatype(.Array~new~identityHash, 'W') \
                 datatype(.StringTable~new~identityHash, 'W') \
                 datatype(.Directory~new~identityHash, 'W')\n\
             a = .Object~new\n\
             b = .Object~new\n\
             say (a~identityHash == a~identityHash) (a~identityHash == b~identityHash)\n"
        ),
        (0, "1 1\n1 1 1 1\n1 0\n".to_string(), String::new())
    );
}

/// Two equal short strings are **one** object here and two on the oracle,
/// which is deviation 4's identity half rather than its rendering half.
#[test]
fn two_equal_inline_strings_share_one_handle() {
    assert_eq!(
        run_source(
            "j = 5\n\
             say ((\"eeeeee\"||j)~identityHash == (\"eeeeee\"||j)~identityHash)\n\
             say ((\"eeeeeee\"||j)~identityHash == (\"eeeeeee\"||j)~identityHash)\n"
        ),
        (0, "1\n0\n".to_string(), String::new())
    );
}

/// The reflection protocol answers on both engines for a receiver that is
/// not a class object, which is where `~class` and `~isA` differ from the
/// `.Class`-scope methods beside them.
#[test]
fn the_object_protocol_answers_a_receiver_that_is_not_a_class_object() {
    assert_eq!(
        run_source(
            "say 'abc'~class~id\n\
             say (12345)~class~id\n\
             say .nil~class~id\n\
             say .Class~superClasses~class~id\n\
             say .Array~package~class~id\n\
             say 'abc'~isA(.String) .nil~isA(.Object) 'abc'~isA(.Array)\n"
        ),
        (
            0,
            "String\nString\nObject\nArray\nPackage\n1 1 0\n".to_string(),
            String::new()
        )
    );
}

/// A name in `.Class`'s own dictionary does not reach a receiver that is
/// not a class object.
#[test]
fn a_class_scope_name_does_not_reach_a_receiver_that_is_not_a_class_object() {
    for source in [
        "say 'abc'~id\n",
        "say .nil~id\n",
        "say .Class~superClasses~id\n",
        "say .Array~package~id\n",
        "say 'abc'~superClasses\n",
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (159, ""), "{source:?}");
        assert!(
            stderr.contains("Error 97.1:"),
            "{source:?} answered or refused instead of raising: {stderr:?}"
        );
    }
}
/// Every class with a `NEW` row of its own answers an instance of the
/// class the send was addressed to, or the oracle's own refusal.
#[test]
fn a_primitive_constructor_answers_an_instance_or_the_oracle_s_own_refusal() {
    for class in [
        "Bag",
        "Directory",
        "EventSemaphore",
        "IdentityTable",
        "List",
        "MutableBuffer",
        "MutexSemaphore",
        "Queue",
        "Relation",
        "Set",
        "Table",
    ] {
        let source = format!("say .{class}~new~class~id\n");
        assert_eq!(
            run_source(&source),
            (0, format!("{class}\n"), String::new()),
            "{class}"
        );
    }
    for (class, status, catalogue) in [
        ("Buffer", 163, "Error 93.967:"),
        ("Class", 163, "Error 93.901:"),
        ("Message", 163, "Error 93.901:"),
        ("Pointer", 163, "Error 93.967:"),
        ("Method", 168, "Error 88.901:"),
        ("Package", 168, "Error 88.901:"),
        ("Routine", 168, "Error 88.901:"),
        ("String", 163, "Error 93.903:"),
        ("Supplier", 163, "Error 93.903:"),
        ("WeakReference", 163, "Error 93.903:"),
    ] {
        let (code, stdout, stderr) = run_source(&format!("say .{class}~new\n"));
        assert_eq!((code, stdout.as_str()), (status, ""), "{class}");
        assert!(stderr.contains(catalogue), "{class}: {stderr:?}");
    }
}

/// The constructors whose argument list carries the instance's whole state
/// answer one, and the state is either kept and read back or refused,
/// never answered from nothing.
#[test]
fn a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state() {
    let message = ".Message~new(.Object~new, 'STRING')";
    assert_eq!(
        run_source(&format!("o = {message}\nsay o~class~id\n")),
        (0, "Message\n".to_string(), String::new())
    );
    let (code, stdout, stderr) = run_source(&format!("o = {message}\nsay o~send\n"));
    assert_eq!((code, stdout.as_str()), (120, ""));
    assert!(stderr.starts_with("rexx-exec: "), "{stderr:?}");
    assert_eq!(
        run_source("o = .MutableBuffer~new('abc')\nsay o~class~id\nsay o~length\n"),
        (0, "MutableBuffer\n3\n".to_string(), String::new())
    );
    // `WeakReference` keeps its referent and reads it back, so it is a
    // readback row rather than a refusal one. That its reference is *weak*
    // is not visible here and cannot be -- nothing on this path collects;
    // `tests/collect_stress.rs` carries that half.
    assert_eq!(
        run_source(
            "k = .Object~new\n\
             o = .WeakReference~new(k)\n\
             say o~class~id (o~value == k) o~value~class~id\n"
        ),
        (0, "WeakReference 1 Object\n".to_string(), String::new())
    );
    // `Supplier` joined the readback group in Phase 5g Task 1. Its `init`
    // used to validate both arrays and drop them, which is the shell this
    // test's refusal rows exist to catch; it now keeps them in the
    // receiver's own pool and walks them.
    assert_eq!(
        run_source(
            "o = .Supplier~new(.Array~of('i'), .Array~of('x'))\n\
             say o~class~id o~available o~item o~index\n"
        ),
        (0, "Supplier 1 i x\n".to_string(), String::new())
    );
    // `say` reaches a `MutableBuffer` through the required-string
    // protocol's `MAKESTRING`, which answers the contents; the
    // receiver-side comparison stays an identity test, which is the
    // oracle's `0` beside the `1` the other operand order answers.
    assert_eq!(
        run_source("say .MutableBuffer~new('abc')\n"),
        (0, "abc\n".to_string(), String::new())
    );
    assert_eq!(
        run_source(
            "buf = .MutableBuffer~new('abc')\n\
             say (buf == 'abc') ('abc' == buf) (buf = 'abc') ('abc' = buf)\n"
        ),
        (0, "0 1 0 1\n".to_string(), String::new())
    );
    // `~result` on a message nothing has sent blocks the oracle, so this
    // is a refusal rather than an answer; `~completed` and `~hasError`
    // beside it are the oracle's own `0`.
    let (code, stdout, stderr) = run_source("say .Message~new(.Object~new, 'STRING')~result\n");
    assert_eq!((code, stdout.as_str()), (120, ""));
    assert_eq!(
        stderr,
        "rexx-exec: `Message~result` on a message whose send has not been made is not \
         implemented (Phase 6)\n"
    );
    assert_eq!(
        run_source(
            "m = .Message~new(.Object~new, 'STRING')\n\
             say m~completed m~hasError\n"
        ),
        (0, "0 0\n".to_string(), String::new())
    );
}

/// A `.Directory~new` reads `.nil` for every index until something puts
/// an entry there, and then reads it back through all three spellings.
#[test]
fn a_new_directory_reads_nil_until_an_entry_is_put_there() {
    assert_eq!(
        run_source(
            "d = .Directory~new\n\
             say d['X'] d~at('X') d~zork\n\
             d~put('v','X')\n\
             say d['X'] d~at('X') d~zork\n\
             d['Y'] = 'w'\n\
             say d['Y'] d~at('Y') d~zork\n\
             d~zork = 'z'\n\
             say d['ZORK'] d~zork d~at('ZORK')\n\
             d2 = .Directory~new\n\
             say d2['zork'] d2~zork\n"
        ),
        (
            0,
            "The NIL object The NIL object The NIL object\n\
             v v The NIL object\n\
             w w The NIL object\n\
             z z z\n\
             The NIL object The NIL object\n"
                .to_string(),
            String::new()
        )
    );
}

/// A `Directory` subclass has a variable pool AND a store, so its
/// `EXPOSE` answers and so do its entry writes.
#[test]
fn a_directory_subclass_keeps_the_instance() {
    assert_eq!(
        run_source(
            "o = .K~new\n\
             say o~class~id o~isA(.Directory) o~peek\n\
             ::class K subclass Directory\n\
             ::method init\n\
             \x20 expose n\n\
             \x20 n = 5\n\
             \x20 self~init:super\n\
             ::method peek\n\
             \x20 expose n\n\
             \x20 return n\n"
        ),
        (0, "K 1 5\n".to_string(), String::new())
    );
    let (code, stdout, stderr) = run_source(
        "o = .Directory~subclass('K')~new\n\
         o['A'] = 1\n\
         say o['A']\n",
    );
    assert_eq!((code, stdout.as_str(), stderr.as_str()), (0, "1\n", ""));
}

/// A stem receiver answers `Stem` and renders as its own value, and a
/// `Stem` method with no body refuses loudly rather than answering.
#[test]
fn a_stem_receiver_answers_stem_and_renders_its_own_value() {
    assert_eq!(
        run_source(
            "s. = 'dflt'\n\
             o = s.\n\
             say o~class~id o~isA(.Stem) o~objectName o~defaultName\n\
             say o o~string\n"
        ),
        (
            0,
            "Stem 1 a Stem a Stem\ndflt dflt\n".to_string(),
            String::new()
        )
    );
    assert_eq!(
        run_source(
            "say '[' || .Stem~new || ']' '[' || .Stem~new('FOO.') || ']'\n\
             say .Stem~new~class~id .Stem~new('FOO.')~string\n"
        ),
        (0, "[] [FOO.]\nStem FOO.\n".to_string(), String::new())
    );
    // **These three used to be the refusals this test pinned**, and
    // Phase 5h Task 5 gave `Stem` its collection surface. Each value is
    // the oracle's: a tail never assigned answers the stem's default,
    // and `items` counts only the tails that hold something.
    for (send, answer) in [("o~at(1)", "dflt"), ("o[1]", "dflt"), ("o~items", "0")] {
        assert_eq!(
            run_source(&format!("s. = 'dflt'\no = s.\nsay {send}\n")),
            (0, format!("{answer}\n"), String::new()),
            "{send}"
        );
    }
    // A subclass would need the body to carry a class of its own, which
    // `Body::Stem` does not, so the constructor refuses rather than
    // answering an object whose `~class~id` is `Stem`.
    let (code, stdout, stderr) = run_source("say .K~new~class~id\n::class K subclass Stem\n");
    assert_eq!((code, stdout.as_str()), (120, ""));
    assert_eq!(
        stderr,
        "rexx-exec: method \"NEW\" of class \"K\" is not implemented (Phase 5)\n"
    );
}

/// A `StringTable` subclass answers its own class and its own method set,
/// and stores what its constructor puts in it -- `.TraceObject~new` is the
/// one the image ships.
#[test]
fn a_string_table_subclass_answers_its_own_class_methods_and_entries() {
    assert_eq!(
        run_source(
            "say .TraceObject~new~class~id .TraceObject~new~hasMethod('makeString')\n\
             say .StringTable~new~class~id .StringTable~new~hasMethod('makeString')\n"
        ),
        (
            0,
            "TraceObject 1\nStringTable 0\n".to_string(),
            String::new()
        )
    );
    assert_eq!(
        run_source(
            "o = .TraceObject~new\n\
             say o['OPTION'] o['NUMBER'] o~at('NUMBER') o['TIMESTAMP']~class~id\n"
        ),
        (0, "N 1 1 DateTime\n".to_string(), String::new())
    );
}

/// A semaphore's `UNINIT` runs at collection rather than at a send, so a
/// class that declares one needs a row even where the finalizer does
/// nothing.
#[test]
fn a_semaphore_instance_runs_its_finalizer_without_refusing() {
    for class in ["EventSemaphore", "MutexSemaphore"] {
        let source = format!("o = .{class}~new\nsay 'built'\n");
        assert_eq!(
            run_source(&source),
            (0, "built\n".to_string(), String::new()),
            "{class}"
        );
    }
}

/// **The one shape of `~at` that has no oracle behaviour to match**, and
/// the instrument [`Loud::array_index_hole`]'s own doc names.
#[test]
fn array_of_fills_its_slots_from_its_arguments() {
    assert_eq!(
        run_source(
            "say .array~of(1,2,3)~size .array~of(1,2,3)~items .array~of(1,2,3)~dimension\n\
             say .array~of()~size .array~of()~items .array~of()~dimension\n\
             say .array~of(1,,3)~size .array~of(1,,3)~items\n\
             say .array~of(1,2,)~size .array~of(1,2,)~items\n\
             say .array~of(4,5)[2] .array~of('x')~class~id\n\
             a = .array~of(7,8)\n\
             a[3] = 9\n\
             say a~size a~toString('l', ' ')\n"
        ),
        (
            0,
            "3 3 1\n0 0 1\n3 2\n2 2\n5 Array\n3 7 8 9\n".to_string(),
            String::new()
        )
    );
}

/// `~of` sent to a subclass of `Array` answers an instance of that
/// subclass -- Phase 5g Task 6, where this test used to assert the
/// refusal.
#[test]
fn array_of_on_a_subclass_answers_an_instance_of_it() {
    assert_eq!(
        run_source("k = .array~subclass('K')\nsay k~of(1,2)~size\nsay k~of(1,2)~class~id\n"),
        (0, "2\nK\n".to_string(), String::new())
    );
}

/// A lone array argument is spread into the subscript list by item count
/// with slot array, so an array whose leading slot is empty and whose item
/// count is one hands the C++ a null subscript and it dies. The program is
/// in `corpus/oracle-crashes.txt` and must never be run against the
/// oracle, so a differential row cannot cover this and this test is the
/// whole of it.
#[test]
fn an_expanded_index_of_one_empty_slot_is_loud() {
    for source in [
        "a = (1,2)\nsay a~at((,2))\n",
        "a = (1,2)\nsay a~at((,,3))\n",
        "a = (1,2)\nsay a[(,2)]\n",
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (120, ""), "{source:?}");
        assert_eq!(
            stderr, "rexx-exec: an array subscript that is an empty slot is not implemented\n",
            "{source:?}"
        );
    }
    // The neighbouring successes: one item spread from a two-slot array,
    // two items spread from a three-slot one, and an empty spread.
    assert_eq!(
        run_source(
            "a = (1,2)\n\
             say a~at((1,))\n\
             say a[(2,)]\n"
        ),
        (0, "1\n2\n".to_string(), String::new())
    );
    let (code, stdout, stderr) = run_source("a = (1,2)\nsay a~at((1,,3))\n");
    assert_eq!((code, stdout.as_str()), (163, ""));
    assert!(stderr.contains("Error 93.926:"), "{stderr:?}");
    let (code, stdout, stderr) = run_source("a = (1,2)\nsay a~at((,))\n");
    assert_eq!((code, stdout.as_str()), (163, ""));
    assert!(stderr.contains("Error 93.901:"), "{stderr:?}");
}

/// The position a subscript refusal names is the subscript's place in the
/// **method's own** argument list, which `putRexx`'s leading value moves
/// by one -- and which index kind the subscript belongs to decides the
/// error as well as the number, because `positionArgument` names no
/// position at all where `requiredPositive` does.
#[test]
fn a_subscript_refusal_names_the_position_its_own_method_counts_from() {
    for (source, message) in [
        (
            "m = .array~new(2,3)\nsay m[,2]\n",
            "Error 93.903:  Missing argument in method; argument 2 is required.",
        ),
        (
            "m = .array~new(2,3)\nm[,2] = 1\n",
            "Error 93.903:  Missing argument in method; argument 3 is required.",
        ),
        (
            "a = (1,2)\na~put('v',0)\n",
            "Error 93.907:  Method argument 2 must be a positive whole number; found \"0\".",
        ),
        (
            "m = .array~new(2,3)\nsay m[1,0]\n",
            "Error 93.924:  Invalid position argument specified; found \"0\".",
        ),
        (
            "say .array~new(2,'-1.0')~size\n",
            "Error 93.906:  Method argument 2 must be zero or a positive whole \
             number; found \"-1.0\".",
        ),
        (
            "say .array~new(100000000000000001)~size\n",
            "Error 93.959:  An array cannot contain more than 100000000000000000 elements.",
        ),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (163, ""), "{source:?}");
        assert!(stderr.contains(message), "{source:?} {stderr:?}");
    }
}
/// A subclass of `Array` constructs, keeps its class, and carries both a
/// store and an object variable pool -- Phase 5g Task 6.
#[test]
fn new_on_a_subclass_of_array_answers_an_instance_of_it() {
    assert_eq!(
        run_source("k = .array~subclass('K')\nsay k~id\nsay k~new(2,3)~size\nsay k~new~class~id\n"),
        (0, "K\n6\nK\n".to_string(), String::new())
    );
}

/// A name a hash collection's behaviour does not answer is **not** 97.1:
/// the forward reaches the receiver's own `UNKNOWN`, whose body reads the
/// name as an entry -- so a missing name is `.nil` and a present one is
/// the entry.
#[test]
fn a_hash_collection_forwards_a_missing_name_to_its_own_unknown() {
    assert_eq!(
        run_source(
            "say .environment~nosuch\n\
             say .local~nosuch\n\
             say .methods~nosuch\n\
             ::method z\n"
        ),
        (
            0,
            "The NIL object\nThe NIL object\nThe NIL object\n".to_string(),
            String::new()
        )
    );
    let (code, stdout, stderr) = run_source("say 'abc'~nosuch\n");
    assert_eq!((code, stdout.as_str()), (159, ""));
    assert!(stderr.contains("Error 97.1:"), "{stderr:?}");
}

/// **The entry-method assignment with no value to store**, which the
/// oracle answers by reading uninitialised memory -- see
/// [`Loud::entry_method_without_a_value`] for the measurement.
#[test]
fn an_entry_method_send_with_no_value_is_loud() {
    for source in [
        ".local~\"MYTHING=\"()\n",
        ".local~\"MYTHING=\"(,)\n",
        ".methods~\"Q=\"()\n::method z\n",
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!((code, stdout.as_str()), (120, ""), "{source:?}");
        assert!(
            stderr.starts_with("rexx-exec: an entry-method assignment to ")
                && stderr.ends_with("with no value is not implemented\n"),
            "{source:?} refused with {stderr:?}"
        );
    }
    assert_eq!(
        run_source(
            ".local~\"MYTHING=\"('v')\n\
             say .MYTHING\n\
             .local~MYTHING = 'w'\n\
             say .MYTHING\n"
        ),
        (0, "v\nw\n".to_string(), String::new())
    );
}

/// **A metaclass carrying its own `NEW` decides what `~subclass` builds**,
/// and this crate has no `NEW` to run -- see [`class_protocol::factory_metaclass`] for why
/// that is a loud refusal rather than a class built from `.Class`'s path.
#[test]
fn a_metaclass_with_its_own_new_is_loud() {
    let (code, stdout, stderr) = run_source(
        "say 'id' .object~subclass(\"k\", .MyMeta)~id\n\
         ::CLASS MyMeta SUBCLASS Class\n\
         ::METHOD new CLASS\n\
         forward class (super)\n",
    );
    assert_eq!((code, stdout.as_str()), (120, ""));
    assert_eq!(
        stderr,
        "rexx-exec: method \"NEW\" of class \"MYMETA\" is not implemented (Phase 5)\n"
    );
    assert_eq!(
        run_source(
            "say 'id' .object~subclass(\"k\", .MyMeta)~id\n\
             ::CLASS MyMeta SUBCLASS Class\n"
        ),
        (0, "id k\n".to_string(), String::new())
    );
}

/// **`.RESOURCES` holds each `::RESOURCE` body as an `Array` of its own
/// lines**, and this is the whole instrument for it.
#[test]
fn the_package_tables_hold_what_their_directives_declare() {
    assert_eq!(
        run_source(
            "say .resources~class\n\
             say .resources~x~class\n\
             say .resources~x~items\n\
             say .resources~x\n\
             say .resources[\"X\"]~at(2)\n\
             say .resources[\"x\"]\n\
             say .resources~q\n\
             ::resource \"x\"\n\
             line one\n\
             line two\n\
             ::END\n"
        ),
        (
            0,
            "The StringTable class\nThe Array class\n2\nline one\nline two\n\
             line two\nThe NIL object\nThe NIL object\n"
                .to_string(),
            String::new()
        )
    );
    assert_eq!(
        run_source(
            "say .resources~x~items\n\
             say '[' || .resources~x || ']'\n\
             ::resource x\n\
             ::END\n"
        ),
        (0, "0\n[]\n".to_string(), String::new())
    );
}

/// An `~UNKNOWN` sent by hand rather than forwarded: its argument list is
/// an `Array` on every forward and anything at all here, and the oracle
/// converts what it is given with `requestArray`.
#[test]
fn an_unknown_sent_by_hand_needs_an_array_this_crate_does_not_convert() {
    assert_eq!(
        run_source("say .environment~unknown('ARRAY', .Array~superClasses)\n"),
        (0, "The Array class\n".to_string(), String::new())
    );
    assert_eq!(
        run_source("say .environment~unknown('ARRAY', 'y')\n"),
        (
            120,
            String::new(),
            "rexx-exec: method \"MAKEARRAY\" of class \"String\" is not implemented \
             (Phase 5)\n"
                .to_string()
        )
    );
}

/// An entry the oracle's own directory holds and this crate does not build
/// is a refusal, not `.nil` -- and the refusal is per directory, because
/// the oracle answers `.nil` for a `.local` name asked of `.environment`.
#[test]
fn a_directory_entry_the_oracle_has_and_this_crate_does_not_is_loud() {
    // **`.environment` has none left**, which is why `.local`'s is the
    // only one asked about. Each read below answers or compares
    // `STDQUE`'s item.
    for source in [
        "say .local['STDQUE']\n",
        "say .context~package~findClass('stdque')\n",
        "say .local~entry('stdque')\n",
        "say .local~stdque\n",
        "say .local~allItems~items\n",
        "say .local~supplier~index\n",
        "say .local~hasItem('x')\n",
        "say .local~index('x')\n",
        "say .local~removeItem('x')\n",
        "say .local~remove('STDQUE')\n",
        "say .local~removeEntry('stdque')\n",
        "do n over .local~allItems; end\n",
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (
                120,
                "",
                "rexx-exec: directory entry \"STDQUE\" is not implemented (Phase 10)\n"
            ),
            "{source:?}"
        );
    }
    assert_eq!(
        run_source("say .stdque\n"),
        (
            120,
            String::new(),
            "rexx-exec: environment symbol \".STDQUE\" is not implemented (Phase 10)\n".to_string()
        )
    );
    // The adjacent successes, each the oracle's own answer: the index and
    // count reads never touch the item, and once a program has replaced
    // the entry every item read answers.
    assert_eq!(
        run_source(
            "say .local~items .local~allIndexes~items .local~hasIndex('STDQUE') \
             .local~hasEntry('stdque')\n\
             .local['STDQUE'] = 'q'\n\
             say .local~allItems~items .stdque .local~index('q')\n"
        ),
        (0, "10 10 1 1\n10 q STDQUE\n".to_string(), String::new())
    );
    assert_eq!(
        run_source(
            "say .environment['STDOUT']\n\
             say .local['ALARM']\n\
             say .environment['ARRAY']~id\n\
             say c2x(.environment['ENDOFLINE'])\n"
        ),
        (
            0,
            "The NIL object\nThe NIL object\nArray\n0A\n".to_string(),
            String::new()
        )
    );
    assert_eq!(
        run_source(
            "d = .local\n\
             d~put('mine', 'STDOUT')\n\
             say d['STDOUT']\n"
        ),
        (0, "mine\n".to_string(), String::new())
    );
}
/// **The required-string protocol answers where it can and refuses where
/// it cannot, and the refusals are only visible here.**
#[test]
fn a_conversion_this_phase_does_not_model_is_loud_where_the_ones_it_models_answer() {
    for (source, message) in [
        // `MAKEARRAY` is in this behaviour's dictionary and this crate
        // has no code for it on a `Directory` built on `NativeObject`'s
        // map. Answering `.nil` would contradict the oracle, which
        // converts: measured, oracle rc 0 and the condition object's
        // array holds `14` items.
        (
            "signal on syntax\nsay 1 + 'a'\nsyntax:\nsay condition('O')~request('ARRAY')~items\n",
            "rexx-exec: method \"MAKEARRAY\" of class \"Directory\" is not implemented \
             (Phase 5)\n",
        ),
        // A receiver with no variable pool to keep a name in. The oracle
        // stores one and remembers it; answering rc 0 and forgetting it
        // would be a wrong answer.
        (
            "'abc'~objectName = 'x'\n",
            "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented \
             (Phase 5)\n",
        ),
        (
            "5~objectName = 'x'\n",
            "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not implemented \
             (Phase 5)\n",
        ),
    ] {
        assert_eq!(
            run_source(source),
            (120, String::new(), message.to_string()),
            "{source:?}"
        );
    }
    // The neighbours, each measured on the oracle: a `MAKE` method the
    // behaviour does not have falls through to the id match and then to
    // `.nil`, and a receiver that does have somewhere to keep a name keeps
    // it.
    for (source, expected) in [
        ("say .K~request('ARRAY')\n::class K\n", "The NIL object\n"),
        ("say .Array~request('CLASS')\n", "The Array class\n"),
        (".K~objectName = 'named'\nsay .K\n::class K\n", "named\n"),
        (
            ".environment~objectName = 'named'\nsay .environment\n",
            "named\n",
        ),
    ] {
        assert_eq!(
            run_source(source),
            (0, expected.to_string(), String::new()),
            "{source:?}"
        );
    }
}

/// **A `reqstr` context whose instruction this phase refuses stays
/// refused**, which is the honest reading of the section's own list: it
/// bounds the row set, and a row whose instruction is another phase's is
/// outside it.
#[test]
fn a_reqstr_context_whose_instruction_is_another_phases_is_still_loud() {
    for (source, message) in [
        (
            "options .K\n::class K\n::method makeString class\n  return 'NOVALUE'\n",
            "rexx-exec: OPTIONS is not implemented (Phase 5)\n",
        ),
        (
            "address 'SYSTEM' .K with output using (.rexxqueue~new)\n::class K\n::method makeString class\n  return 'true'\n",
            "rexx-exec: an ADDRESS WITH RexxQueue redirection is not implemented (Phase 10)\n",
        ),
    ] {
        assert_eq!(
            run_source(source),
            (120, String::new(), message.to_string()),
            "{source:?}"
        );
    }
}

/// An `Interp` whose class registry holds one class awaiting a class-side
/// `UNINIT`, which is what a directive install leaves behind.
fn interp_awaiting_a_class_uninit() -> Interp {
    let mut interp = Interp::new();
    let program = Rc::new(
        rexx_parse::parse_program(b"nop\n::class k\n::method uninit class\n  nop\n".to_vec())
            .expect("it parses"),
    );
    // The id `install_directives` is given has to name a program in
    // `Interp::programs`, which is what `Interp::run` pushes before it
    // installs anything; without it a finalizer's body cannot be reached.
    let id = crate::ProgramId(interp.programs.len());
    interp.programs.push(Rc::clone(&program));
    interp
        .install_directives(id, &program)
        .expect("the directives install");
    interp
}

/// [`Interp::run_termination_uninits`] refuses to re-enter itself, and
/// refusing leaves its work pending rather than consuming it.
#[test]
fn an_interlocked_termination_sweep_runs_nothing_and_consumes_nothing() {
    let mut fresh = interp_awaiting_a_class_uninit();
    assert_eq!(
        fresh.classes().take_uninit_classes_in_sweep_order().len(),
        1,
        "the setup must really leave a class pending, or both arms below \
         are green over an empty registry"
    );

    let mut held = interp_awaiting_a_class_uninit();
    held.processing_uninits = true;
    assert!(
        held.run_termination_uninits().is_empty(),
        "an interlocked sweep answers no loud refusal because it runs nothing"
    );
    assert_eq!(
        held.classes().take_uninit_classes_in_sweep_order().len(),
        1,
        "an interlocked sweep must leave the class for the caller holding \
         the flag; an empty answer here means it swept anyway"
    );

    let mut free = interp_awaiting_a_class_uninit();
    assert!(free.run_termination_uninits().is_empty());
    assert!(
        free.classes()
            .take_uninit_classes_in_sweep_order()
            .is_empty(),
        "a sweep that is not interlocked consumes what it ran"
    );
}
