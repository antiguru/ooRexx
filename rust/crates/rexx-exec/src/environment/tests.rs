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

/// The article rule, both ways round, because a version that always says
/// `a ` renders `a Array` and one that always says `an ` renders `an
/// StringTable`.
#[test]
fn the_default_object_name_picks_its_article_by_the_ids_first_letter() {
    assert_eq!(default_object_name("StringTable"), "a StringTable");
    assert_eq!(default_object_name("Array"), "an Array");
    assert_eq!(default_object_name("RexxContext"), "a RexxContext");
}

/// Every name the oracle's `.environment` and `.local` hold either
/// resolves here or fails loudly -- never the dotted-text fallback, which for one of these
/// names would be a silent wrong answer at rc 0.
#[test]
fn every_name_the_oracle_answers_resolves_or_is_loud() {
    let mut interp = Interp::new();
    for name in ORACLE_ENVIRONMENT.iter().chain(ORACLE_LOCAL.iter()) {
        let dotted = format!(".{name}").into_bytes();
        match interp.dot_variable(&dotted) {
            Err(Failure::Loud(_)) => {}
            Err(other) => panic!(".{name} failed with {other:?} rather than loudly"),
            Ok(value) => {
                let text = interp.to_text(value).to_vec();
                assert_ne!(
                    text, dotted,
                    ".{name} fell back to its own text, which is what the \
                     oracle answers only for a name it does not resolve"
                );
            }
        }
    }
    let undefined = interp
        .dot_variable(b".FUNNYCONST")
        .expect("a name neither directory holds falls back rather than failing");
    assert_eq!(interp.to_text(undefined).to_vec(), b".FUNNYCONST".to_vec());
}

/// The environment holds the classes the registry registered, and the
/// entries `addToEnvironment` adds by hand.
#[test]
fn the_environment_holds_the_native_classes_and_the_hand_registered_entries() {
    let mut interp = Interp::new();
    let array = interp.classes().lookup("Array").expect("Array is native");
    assert_eq!(interp.dot_variable(b".ARRAY").expect("resolves"), array);
    assert_eq!(interp.dot_variable(b".NIL").expect("resolves"), ObjRef::NIL);
    let true_value = interp.dot_variable(b".TRUE").expect("resolves");
    assert_eq!(interp.to_text(true_value).to_vec(), b"1".to_vec());
    let environment = interp.dot_variable(b".ENVIRONMENT").expect("resolves");
    assert_eq!(
        interp.to_text(environment).to_vec(),
        b"The Environment Directory".to_vec()
    );
    // The rendering is an assigned `~objectName` and says nothing about
    // what the object is; its class is what makes it a directory rather
    // than a labelled blob, so both are asserted.
    let directory = interp
        .classes()
        .lookup("Directory")
        .expect("Directory is native");
    let object = interp.heap.get(environment).expect("a rooted directory");
    let Body::Instance { class, .. } = &object.body else {
        panic!("the environment is a Directory instance with a store")
    };
    assert_eq!(*class, directory);
    let local = interp.dot_variable(b".LOCAL").expect("resolves");
    assert_eq!(
        interp.to_text(local).to_vec(),
        b"The Local Directory".to_vec()
    );
    // `.environment` holds itself, which is what makes the entry above a
    // registration rather than a rendering special case.
    assert_eq!(
        interp.dot_variable(b".ENVIRONMENT").expect("resolves"),
        environment
    );
    assert_ne!(environment, local);
}
/// The names and order the bootstrap leaves in both directories are the
/// oracle's, and the only entry still owed is `.local`'s `STDQUE`.
#[test]
fn the_bootstrap_leaves_the_oracles_names_in_the_oracles_order() {
    let mut interp = Interp::new();
    interp
        .bootstrap_library()
        .expect("the library bootstrap runs");
    for (dotted, oracle) in [
        (b".ENVIRONMENT".as_slice(), ORACLE_ENVIRONMENT),
        (b".LOCAL".as_slice(), ORACLE_LOCAL),
    ] {
        let directory = interp.dot_variable(dotted).expect("resolves");
        let caller = interp.caller();
        let indexes = interp
            .send_message(directory, b"ALLINDEXES", None, &[], caller)
            .expect("answers")
            .expect("answers a value");
        let names: Vec<String> = interp
            .array_slots_of(indexes)
            .expect("an array")
            .into_iter()
            .flatten()
            .map(|index| String::from_utf8_lossy(&interp.to_text(index)).into_owned())
            .collect();
        assert_eq!(names, oracle, "{}", String::from_utf8_lossy(dotted));
        let owed: Vec<&str> = oracle
            .iter()
            .copied()
            .filter(|name| {
                matches!(
                    hash::directory_get(
                        &mut interp,
                        directory,
                        hash::Key::new(name.as_bytes()),
                        None
                    ),
                    Ok((hash::DirectoryEntry::Owed(_), _))
                )
            })
            .collect();
        let expected: &[&str] = if oracle == ORACLE_LOCAL {
            &["STDQUE"]
        } else {
            &[]
        };
        assert_eq!(owed, expected, "{}", String::from_utf8_lossy(dotted));
    }
}

/// **Every method `Directory`'s own table holds answers on `.environment`
/// and on `.local` what it answers on a `.Directory~new` holding the same
/// entries in the same order**, enumerated from the registry rather than
/// listed: each name resolves to the method a new `Directory` resolves to,
/// and the same sends, in the same order, print the same answers on both.
#[test]
fn every_directory_method_answers_on_both_directories_as_on_a_copy() {
    let mut interp = Interp::new();
    interp
        .bootstrap_library()
        .expect("the library bootstrap runs");
    let class = interp
        .classes()
        .lookup("Directory")
        .expect("Directory is native");
    let names = interp.classes().own_instance_method_names(class);
    assert!(
        names.contains("HASENTRY") && names.contains("[]="),
        "the registry's Directory table is not the one this test is about: {names:?}"
    );
    let fresh = interp.classes().instance_behaviour_handle(class);
    // `EMPTY` last, since it takes the contents out.
    let mut ordered: Vec<&String> = names.iter().filter(|name| *name != "EMPTY").collect();
    ordered.extend(names.iter().filter(|name| *name == "EMPTY"));
    let mut sends = String::new();
    for name in ordered {
        let arguments = match name.as_str() {
            "ALLINDEXES" | "ALLITEMS" | "EMPTY" | "INIT" | "ISEMPTY" | "ITEMS" | "MAKEARRAY"
            | "SUPPLIER" => "",
            "AT" | "[]" | "ENTRY" | "HASENTRY" | "HASINDEX" | "REMOVE" | "REMOVEENTRY"
            | "UNSETMETHOD" => "'ZZ'",
            "HASITEM" | "INDEX" | "REMOVEITEM" => "'zz value'",
            "PUT" | "[]=" => "'zz value', 'ZZ'",
            "SETENTRY" => "'ZZ', 'zz value'",
            "SETMETHOD" => "'ZZ', 'return \"zz method\"'",
            "UNKNOWN" => "'ZZ', .array~new",
            other => panic!("Directory's table holds {other}, which this test has no call for"),
        };
        for dotted in [b".ENVIRONMENT".as_slice(), b".LOCAL".as_slice()] {
            let receiver = interp.dot_variable(dotted).expect("resolves");
            let Some(Body::Instance { behaviour, .. }) =
                interp.heap.get(receiver).map(|object| &object.body)
            else {
                panic!(
                    "{} is not a Directory instance",
                    String::from_utf8_lossy(dotted)
                );
            };
            let behaviour = *behaviour;
            assert_eq!(
                interp.classes().lookup_at(behaviour, name),
                interp.classes().lookup_at(fresh, name),
                "{}~{name} resolves to a different method than a new Directory's",
                String::from_utf8_lossy(dotted)
            );
        }
        sends.push_str(&format!(
            "r['ZZ'] = 'zz value'\ndrop result\nr~\"{name}\"({arguments})\n\
             if var('RESULT') then say '{name}' show(result, r)\n\
             else say '{name} answers nothing'\n"
        ));
    }
    let report = "say 'final' r~items show(r~allIndexes, r)\nexit 0\n\
                  show: procedure\n\
                  \x20 use arg value, receiver\n\
                  \x20 if receiver == value then return 'the receiver'\n\
                  \x20 if value~isA(.Array) then return 'array' value~items value~makeString('L', '|')\n\
                  \x20 if value~isA(.Supplier) then do\n\
                  \x20   out = 'supplier'\n\
                  \x20   do while value~available\n\
                  \x20     out = out value~index'='value~item~string\n\
                  \x20     value~next\n\
                  \x20   end\n\
                  \x20   return out\n\
                  \x20 end\n\
                  \x20 return value~string\n";
    for (directory, oracle, capacity, prepare) in [
        ("environment", ORACLE_ENVIRONMENT, ENVIRONMENT_CAPACITY, ""),
        ("local", ORACLE_LOCAL, 0, ".local['STDQUE'] = 'q'\n"),
    ] {
        // The copy is filled from the oracle's list at the directory's own
        // size, each value read as `.NAME` rather than by a send to the
        // directory, and `LOCAL` -- `.environment`'s method-table entry --
        // goes in as a method.
        let contents: Vec<&str> = oracle
            .iter()
            .copied()
            .filter(|name| *name != "LOCAL")
            .collect();
        let method = if oracle.contains(&"LOCAL") {
            "r~setMethod('LOCAL', 'return .local')\n"
        } else {
            ""
        };
        let copy = format!(
            "list = '{}'\n\
             r = .Directory~new({capacity})\n\
             do i = 1 to words(list)\n\
             \x20 name = word(list, i)\n\
             \x20 if name == 'STDQUE' then r[name] = 'q'\n\
             \x20 else r[name] = value('.'name)\n\
             end\n\
             {method}",
            contents.join(" ")
        );
        let original = format!("{prepare}r = .{directory}\n");
        let mut transcripts = Vec::new();
        for setup in [original, copy] {
            let source = format!("{setup}{sends}{report}");
            let outcome =
                crate::run_program("/t.rex", source.into_bytes(), crate::Invocation::none());
            let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
            assert_eq!(outcome.exit_code, 0, ".{directory}: {stderr}");
            assert!(stderr.is_empty(), ".{directory}: {stderr}");
            transcripts.push(String::from_utf8_lossy(&outcome.stdout).into_owned());
        }
        assert!(
            transcripts[1].lines().count() > names.len(),
            ".{directory}'s copy printed too little to compare: {}",
            transcripts[1]
        );
        assert_eq!(
            transcripts[0], transcripts[1],
            ".{directory} answered differently from a Directory holding its entries"
        );
    }
}
