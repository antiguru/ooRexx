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

//! The conversion table, row by row and against the frozen header.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use rexx_api::handles::Table;
use rexx_api::layout::POINTER;
use rexx_api::values::{
    ARGUMENT_EXISTS, CStringPool, Constants, Conversion, Direction, Failure, Host,
    OPTIONAL_ARGUMENT, Repr, SPECIAL_ARGUMENT, Value, code, consumes_argument, descriptor,
    from_native, repr, rows, to_native,
};
use rexx_core::{BehaviourHandle, Body, Bytes, Heap, ObjRef, RootSet};

/// Stands in for the interpreter the table calls back into.
///
/// `string_value` models `requiredString`: a text object is its own string
/// value, a small integer has one that must be built, and anything in
/// `speechless` has none.
struct Interpreter {
    heap: Heap,
    method: bool,
    cself: Option<POINTER>,
    speechless: Vec<ObjRef>,
    variables: Vec<(Vec<u8>, ObjRef)>,
    locals: Table,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            heap: Heap::new(),
            method: true,
            cself: None,
            speechless: Vec::new(),
            variables: Vec::new(),
            locals: Table::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> ObjRef {
        self.heap.alloc(Body::Text {
            bytes: Bytes::from_slice(bytes),
            num: None,
        })
    }
}

impl Host for Interpreter {
    fn is_method(&self) -> bool {
        self.method
    }

    fn string_value(&mut self, object: ObjRef) -> Option<ObjRef> {
        if self.speechless.contains(&object) {
            return None;
        }
        match object.decode() {
            rexx_core::Decoded::SmallInt(number) => Some(self.text(number.to_string().as_bytes())),
            rexx_core::Decoded::Text(_) => Some(object),
            _ => match self.heap.get(object)?.body {
                Body::Text { .. } => Some(object),
                _ => None,
            },
        }
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        if let rexx_core::Decoded::Text(inline) = object.decode() {
            return Some(Cow::Owned(inline.to_vec()));
        }
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        self.cself
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        Constants {
            nil: ObjRef::NIL,
            true_object: ObjRef::small_int(1).expect("one is a small integer"),
            false_object: ObjRef::small_int(0).expect("zero is a small integer"),
            null_string: self.text(b""),
        }
    }

    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>) {
        let name = name.to_ascii_uppercase();
        self.variables.retain(|(bound, _)| *bound != name);
        if let Some(value) = value {
            self.variables.push((name, value));
        }
    }

    fn drop_object_variable(&mut self, name: &[u8]) {
        self.set_object_variable(name, None);
    }

    fn whole_number(&mut self, value: isize) -> ObjRef {
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_pointer(&mut self, value: POINTER) -> ObjRef {
        let body = Body::pointer(ObjRef::NIL, BehaviourHandle::new(0), value);
        self.heap.alloc(body)
    }

    fn locals(&mut self) -> &mut Table {
        &mut self.locals
    }
}

/// The roots a collection sees when the local-reference table is the only
/// thing holding an object.
fn roots_of(locals: &Table) -> RootSet {
    let mut roots = RootSet::new();
    for (index, object) in locals.roots().enumerate() {
        roots.add_global(&format!(".HANDLE{index}"), object);
    }
    roots
}

/// The frozen header (D5).
fn header() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../api/oorexxapi.h");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every `#define REXX_VALUE_<name> <number>` the header carries, minus the
/// platform aliases, whose names begin with an underscore.
fn header_codes() -> BTreeMap<String, u16> {
    let mut found = BTreeMap::new();
    for line in header().lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("#define REXX_VALUE_") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let (Some(name), Some(number)) = (parts.next(), parts.next()) else {
            continue;
        };
        if name.starts_with('_') {
            continue;
        }
        let Ok(code) = number.parse::<u16>() else {
            continue;
        };
        found.insert(name.to_string(), code);
    }
    found
}

// ---------------------------------------------------------------- the table

#[test]
fn the_table_has_a_row_for_every_code_the_header_defines() {
    let table: BTreeMap<String, u16> = rows()
        .map(|(code, name)| (name.to_string(), code))
        .collect();
    assert_eq!(
        table,
        header_codes(),
        "the table and `api/oorexxapi.h` disagree about the REXX_VALUE_ codes"
    );
}

/// A platform alias is a second spelling of a code the table already has, not
/// a row of its own.
#[test]
fn every_platform_alias_names_a_code_the_table_has() {
    let known: Vec<u16> = rows().map(|(code, _)| code).collect();
    let mut aliases = 0;
    for line in header().lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("#define REXX_VALUE__") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let (Some(_), Some(number)) = (parts.next(), parts.next()) else {
            continue;
        };
        let Ok(code) = number.parse::<u16>() else {
            continue;
        };
        aliases += 1;
        assert!(known.contains(&code), "alias for an unknown code {code}");
    }
    assert!(
        aliases > 0,
        "the scan found no aliases, so it is not reading the header"
    );
}

/// `OPTIONAL_CSTRING` is `CSTRING` with a bit set, which is why it is not a
/// row of its own. Asserted over every `REXX_VALUE_OPTIONAL_` line rather
/// than for the one this task needs.
#[test]
fn every_optional_name_is_a_base_code_with_the_optional_bit() {
    let codes = header_codes();
    let mut optionals = 0;
    for line in header().lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("#define REXX_VALUE_OPTIONAL_") else {
            continue;
        };
        let Some((name, body)) = rest.split_once(char::is_whitespace) else {
            continue;
        };
        optionals += 1;
        let base = *codes
            .get(name)
            .unwrap_or_else(|| panic!("OPTIONAL_{name} has no base code"));
        assert_eq!(
            body.split_whitespace().collect::<String>(),
            format!("(REXX_OPTIONAL_ARGUMENT|REXX_VALUE_{name})"),
            "OPTIONAL_{name} is not its base code with the optional bit"
        );
        assert_eq!(
            rexx_api::values::argument_type(OPTIONAL_ARGUMENT | base),
            base
        );
        assert!(rexx_api::values::is_optional(OPTIONAL_ARGUMENT | base));
    }
    assert!(
        optionals > 0,
        "the scan found no REXX_VALUE_OPTIONAL_ lines, so it is not reading the header"
    );
}

/// Each row's union member, checked against the C type the header's own
/// `ARGUMENT_TYPE_<name>` define gives that code. Reading the union is
/// `ffi.rs`'s work, and this is what lets it read one without a second switch
/// on the code.
#[test]
fn every_row_names_the_union_member_the_header_gives_its_code() {
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    for line in header().lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some(rest) = line.strip_prefix("#define ARGUMENT_TYPE_") else {
            continue;
        };
        let Some((name, c_type)) = rest.split_once(char::is_whitespace) else {
            continue;
        };
        declared.insert(name.to_string(), c_type.trim().to_string());
    }
    assert!(
        declared.contains_key("CSELF"),
        "the scan found no ARGUMENT_TYPE_ defines, so it is not reading the header"
    );

    let expected = |c_type: &str| -> Repr {
        match c_type {
            "CSTRING" => Repr::CString,
            "POINTER" => Repr::Pointer,
            "int" => Repr::Int,
            "int8_t" => Repr::Int8,
            "int16_t" => Repr::Int16,
            "int32_t" => Repr::Int32,
            "int64_t" => Repr::Int64,
            "uint8_t" => Repr::Uint8,
            "uint16_t" => Repr::Uint16,
            "uint32_t" => Repr::Uint32,
            "uint64_t" => Repr::Uint64,
            "wholenumber_t" | "ssize_t" | "intptr_t" => Repr::Isize,
            "stringsize_t" | "size_t" | "uintptr_t" | "logical_t" => Repr::Usize,
            "double" => Repr::Double,
            "float" => Repr::Float,
            other if other.starts_with("Rexx") => Repr::Object,
            other => panic!("no union member is known for the C type {other}"),
        }
    };

    for (code, name) in rows() {
        let c_type = declared
            .get(name)
            .unwrap_or_else(|| panic!("the header has no ARGUMENT_TYPE_{name}"));
        assert_eq!(
            repr(code),
            Some(expected(c_type)),
            "REXX_VALUE_{name} is declared {c_type}"
        );
        assert_eq!(
            repr(OPTIONAL_ARGUMENT | code),
            repr(code),
            "the optional bit must not change the union member"
        );
    }
    assert_eq!(repr(9999), None);
}

/// The rows that take their value from the context rather than the argument
/// list, asserted as a set.
#[test]
fn the_special_rows_are_the_ones_that_consume_no_argument() {
    let special: Vec<u16> = rows()
        .map(|(code, _)| code)
        .filter(|code| consumes_argument(*code) == Some(false))
        .collect();
    assert_eq!(
        special,
        vec![
            code::ARGLIST,
            code::NAME,
            code::SCOPE,
            code::CSELF,
            code::OSELF,
            code::SUPER
        ]
    );
    assert_eq!(consumes_argument(9999), None);
}

/// Which rows have a conversion, as a set rather than one name at a time. A
/// row that stops converting, or one that starts, changes this list.
#[test]
fn exactly_the_filled_rows_convert() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };

    let mut inbound = Vec::new();
    let mut outbound = Vec::new();
    for (code, _) in rows() {
        fn unfilled<T>(result: &Result<T, Failure>) -> bool {
            matches!(result, Err(Failure::Unfilled { .. }))
        }
        if !unfilled(&to_native(&mut cx, code, Some(subject), 1)) {
            inbound.push(code);
        }
        if !unfilled(&from_native(
            &mut cx,
            code,
            Value::Object(std::ptr::null_mut()),
        )) {
            outbound.push(code);
        }
    }
    assert_eq!(
        inbound,
        vec![code::CSELF, code::CSTRING, code::REXX_STRING_OBJECT]
    );
    assert_eq!(
        outbound,
        vec![code::INT, code::POINTER, code::REXX_STRING_OBJECT]
    );
}

/// The refusal a row this phase has not written gives, and the phase it names.
#[test]
fn an_unfilled_row_refuses_and_names_its_code() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let refusal = to_native(&mut cx, code::REXX_ARRAY_OBJECT, Some(subject), 1)
        .expect_err("no row converts an array yet");
    assert_eq!(
        refusal,
        Failure::Unfilled {
            code: code::REXX_ARRAY_OBJECT,
            name: "RexxArrayObject",
            direction: Direction::ToNative,
        }
    );
    assert_eq!(refusal.error_number(true), None);
    assert!(refusal.to_string().contains("Phase 8"));
}

/// A code the header does not define is a signature error, which is the
/// oracle's `default` in both switches.
#[test]
fn a_code_the_table_does_not_know_is_a_signature_error() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, 9999, None, 1),
        Err(Failure::Signature),
        "an unknown code must not be read as a missing argument"
    );
    assert_eq!(
        from_native(&mut cx, 9999, Value::Int(0)),
        Err(Failure::Signature)
    );
}

/// A signature error's number differs by context.
#[test]
fn the_signature_error_is_93_968_in_a_method_and_40_918_in_a_call() {
    assert_eq!(Failure::Signature.error_number(true), Some(93968));
    assert_eq!(Failure::Signature.error_number(false), Some(40918));
}

/// A zero type is an omitted value rather than a bad one
/// (`NativeActivation.cpp:848`).
#[test]
fn the_terminator_converts_to_no_object() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(
            &mut cx,
            rexx_api::values::ARGUMENT_TERMINATOR,
            Value::Omitted
        ),
        Ok(None)
    );
}

// -------------------------------------------------------------- the CSTRING

#[test]
fn the_cstring_row_points_at_a_copy_of_the_argument() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, code::CSTRING, Some(subject), 1).expect("a string converts");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let Value::CString(pointer) = converted.value else {
        panic!("a CSTRING row answers a CSTRING, got {:?}", converted.value)
    };
    assert_eq!(
        strings.bytes_at(pointer),
        Some(&b"a string long enough to reach the heap"[..])
    );
}

/// The property section 4 of the spec names: the bytes are reachable for the
/// whole call, so a collection between the conversion and the use does not
/// take them away.
#[test]
fn a_cstring_outlives_a_collection_and_the_object_it_came_from() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let pointer = {
        let mut cx = Conversion {
            host: &mut host,
            strings: &mut strings,
        };
        let converted =
            to_native(&mut cx, code::CSTRING, Some(subject), 1).expect("a string converts");
        match converted.value {
            Value::CString(pointer) => pointer,
            other => panic!("a CSTRING row answers a CSTRING, got {other:?}"),
        }
    };

    // Nothing roots the argument, so this is the collection that would free
    // the bytes if the pointer went into the heap.
    let swept = host.heap.collect(&RootSet::new());
    assert_eq!(swept.swept, 1, "the argument must actually be reclaimed");
    assert!(host.heap.get(subject).is_none());

    // Later conversions must not move an already minted pointer either.
    let mut later = CStringPool::new();
    std::mem::swap(&mut later, &mut strings);
    let mut strings = later;
    for _ in 0..64 {
        strings.intern(b"pressure on the pool");
    }
    assert_eq!(
        strings.bytes_at(pointer),
        Some(&b"a string long enough to reach the heap"[..])
    );
}

/// Ending the call is what ends a `CSTRING`'s lifetime, and nothing before it.
#[test]
fn clearing_the_pool_drops_the_copies() {
    let mut strings = CStringPool::new();
    let pointer = strings.intern(b"kept until the call ends");
    assert_eq!(strings.len(), 1);
    strings.clear();
    assert!(strings.is_empty());
    assert_eq!(strings.bytes_at(pointer), None);
}

/// Bytes are copied verbatim, so a Rexx string with an embedded zero is
/// truncated from the extension's side rather than rejected.
#[test]
fn an_embedded_zero_is_copied_rather_than_refused() {
    let mut strings = CStringPool::new();
    let pointer = strings.intern(b"before\0after");
    assert_eq!(strings.bytes_at(pointer), Some(&b"before\0after"[..]));
}

#[test]
fn a_required_cstring_that_is_absent_is_88_901() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let refusal =
        to_native(&mut cx, code::CSTRING, None, 1).expect_err("a required argument is required");
    assert_eq!(refusal, Failure::MissingArgument { position: 1 });
    assert_eq!(refusal.error_number(true), Some(88901));
    assert_eq!(refusal.error_number(false), Some(88901));
}

#[test]
fn a_required_cstring_with_no_string_value_is_88_909() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    host.speechless.push(subject);
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let refusal = to_native(&mut cx, code::CSTRING, Some(subject), 2)
        .expect_err("an object with no string value cannot convert");
    assert_eq!(refusal, Failure::NoStringValue { position: 2 });
    assert_eq!(refusal.error_number(true), Some(88909));
}

/// The direction the surface half still owes for this row.
#[test]
fn a_cstring_returned_by_an_extension_is_not_converted_yet() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, code::CSTRING, Value::CString(std::ptr::null())),
        Err(Failure::Unfilled {
            code: code::CSTRING,
            name: "CSTRING",
            direction: Direction::FromNative,
        })
    );
}

// ----------------------------------------------------- the OPTIONAL_CSTRING

#[test]
fn an_omitted_optional_cstring_is_a_zero_with_no_flags() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, OPTIONAL_ARGUMENT | code::CSTRING, None, 1)
        .expect("an optional argument may be left off");
    assert_eq!(converted.value, Value::Omitted);
    assert_eq!(converted.flags, 0);
    assert!(strings.is_empty(), "an omitted argument interns nothing");
}

#[test]
fn an_optional_cstring_that_is_supplied_converts_like_a_required_one() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let required = to_native(&mut cx, code::CSTRING, Some(subject), 1).expect("required converts");
    let optional = to_native(&mut cx, OPTIONAL_ARGUMENT | code::CSTRING, Some(subject), 1)
        .expect("optional converts");
    assert_eq!(required.flags, optional.flags);
    let (Value::CString(a), Value::CString(b)) = (required.value, optional.value) else {
        panic!("both are CSTRINGs")
    };
    assert_eq!(strings.bytes_at(a), strings.bytes_at(b));
    assert_ne!(a, b, "each conversion gets its own copy");
}

/// An omitted optional argument is answered before the per-type conversion,
/// so a row whose conversion is not written yet still handles one.
#[test]
fn an_omitted_optional_argument_does_not_need_its_row_filled() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, OPTIONAL_ARGUMENT | code::INT, None, 1)
        .expect("an omitted optional needs no conversion");
    assert_eq!(converted.value, Value::Omitted);
    assert_eq!(converted.flags, 0);
    assert_eq!(
        to_native(&mut cx, code::INT, None, 1),
        Err(Failure::MissingArgument { position: 1 }),
        "a required one is still missing rather than unfilled"
    );
}

/// The header marks the variable-reference code as never optional, and the
/// C++ leaves it out of the absent switch, so an omitted one is a signature
/// error rather than a zero.
#[test]
fn an_omitted_optional_variable_reference_is_a_signature_error() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(
            &mut cx,
            OPTIONAL_ARGUMENT | code::REXX_VARIABLE_REFERENCE_OBJECT,
            None,
            1
        ),
        Err(Failure::Signature)
    );
}

// ----------------------------------------------------------------- the CSELF

#[test]
fn the_cself_row_reads_the_seam_and_consumes_no_argument() {
    let mut host = Interpreter::new();
    let block = std::ptr::without_provenance_mut::<std::ffi::c_void>(0xC5E1F);
    host.cself = Some(block);
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, code::CSELF, None, 1).expect("CSELF needs no argument");
    assert_eq!(converted.value, Value::Pointer(block));
    assert_eq!(converted.flags, ARGUMENT_EXISTS | SPECIAL_ARGUMENT);
}

/// `NativeActivation::cself` answers NULL when the object variable is not
/// set, and does not raise.
#[test]
fn a_receiver_with_no_cself_variable_converts_to_null() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted =
        to_native(&mut cx, code::CSELF, None, 1).expect("an unset CSELF is not an error");
    assert_eq!(converted.value, Value::Pointer(std::ptr::null_mut()));
}

#[test]
fn cself_outside_a_method_is_a_signature_error() {
    let mut host = Interpreter::new();
    host.method = false;
    host.cself = Some(std::ptr::without_provenance_mut(0xC5E1F));
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let refusal = to_native(&mut cx, code::CSELF, None, 1).expect_err("a call has no CSELF");
    assert_eq!(refusal, Failure::Signature);
    assert_eq!(refusal.error_number(false), Some(40918));
}

/// Minting the `.Pointer` a returned CSELF would need is Task 7's.
#[test]
fn a_cself_returned_by_an_extension_is_not_converted_yet() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, code::CSELF, Value::Pointer(std::ptr::null_mut())),
        Err(Failure::Unfilled {
            code: code::CSELF,
            name: "CSELF",
            direction: Direction::FromNative,
        })
    );
}

// ------------------------------------------------------ the RexxStringObject

#[test]
fn the_string_object_row_round_trips_through_a_handle() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted =
        to_native(&mut cx, code::REXX_STRING_OBJECT, Some(subject), 1).expect("a string converts");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    assert_eq!(
        converted.value,
        Value::Object(cx.host.locals().register(subject))
    );
    assert_eq!(
        from_native(&mut cx, code::REXX_STRING_OBJECT, converted.value),
        Ok(Some(subject))
    );
}

/// The conversion that has to build a string is the one whose result nothing
/// else holds, and registering it is what keeps it alive.
#[test]
fn a_string_the_conversion_built_is_rooted_for_the_call() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let number = ObjRef::small_int(42).expect("42 is small");
    let handle = {
        let mut cx = Conversion {
            host: &mut host,
            strings: &mut strings,
        };
        let converted = to_native(&mut cx, code::REXX_STRING_OBJECT, Some(number), 1)
            .expect("a number has a string value");
        match converted.value {
            Value::Object(handle) => handle,
            other => panic!("a string object row answers a handle, got {other:?}"),
        }
    };

    let roots = roots_of(host.locals());
    host.heap.collect(&roots);
    let built = host
        .locals()
        .resolve(handle)
        .expect("the table still holds it");
    assert_eq!(host.string_bytes(built).as_deref(), Some(&b"42"[..]));
}

/// The control for the test above: without the table in the root set the
/// built string is swept, so the registration is what the survival rests on.
#[test]
fn the_same_string_is_swept_when_the_table_is_not_a_root() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let number = ObjRef::small_int(42).expect("42 is small");
    let handle = {
        let mut cx = Conversion {
            host: &mut host,
            strings: &mut strings,
        };
        match to_native(&mut cx, code::REXX_STRING_OBJECT, Some(number), 1)
            .expect("a number has a string value")
            .value
        {
            Value::Object(handle) => handle,
            other => panic!("a string object row answers a handle, got {other:?}"),
        }
    };

    let swept = host.heap.collect(&RootSet::new());
    assert_eq!(swept.swept, 1);
    let built = host
        .locals()
        .resolve(handle)
        .expect("the table still holds the handle");
    assert_eq!(host.string_bytes(built), None, "the object is gone");
}

#[test]
fn a_required_string_object_that_is_absent_is_88_901() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, code::REXX_STRING_OBJECT, None, 3),
        Err(Failure::MissingArgument { position: 3 })
    );
}

#[test]
fn a_string_object_with_no_string_value_is_88_909() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    host.speechless.push(subject);
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, code::REXX_STRING_OBJECT, Some(subject), 1),
        Err(Failure::NoStringValue { position: 1 })
    );
}

/// A handle the activation no longer holds is a miss, not a hit on whatever
/// took the slot (D5).
#[test]
fn a_handle_the_table_dropped_does_not_convert_back() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let handle = host.locals().register(subject);
    host.locals().clear();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, code::REXX_STRING_OBJECT, Value::Object(handle)),
        Err(Failure::StaleHandle)
    );
}

/// `NULLOBJECT` is the null pointer, and the oracle passes it through as
/// "no result" rather than as an object.
#[test]
fn a_null_handle_converts_to_no_object() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(
            &mut cx,
            code::REXX_STRING_OBJECT,
            Value::Object(std::ptr::null_mut())
        ),
        Ok(None)
    );
}

// ------------------------------------------------------------------- the int

#[test]
fn the_int_row_converts_a_returned_value() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    for number in [0, 1, -1, 3, i32::MAX, i32::MIN] {
        assert_eq!(
            from_native(&mut cx, code::INT, Value::Int(number)),
            Ok(ObjRef::small_int(i64::from(number))),
            "converting {number}"
        );
    }
}

/// A value that is not the row's own is a signature error, which is what
/// keeps a mismatched descriptor from being read as a plausible number.
#[test]
fn the_int_row_refuses_a_value_of_another_type() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, code::INT, Value::Omitted),
        Err(Failure::Signature)
    );
}

/// `int` is a return type in the extension this slice loads, so the argument
/// direction is a row the surface half still owes.
#[test]
fn an_int_argument_is_not_converted_yet() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, code::INT, Some(subject), 1),
        Err(Failure::Unfilled {
            code: code::INT,
            name: "int",
            direction: Direction::ToNative,
        })
    );
}

// ------------------------------------------------------------ the descriptor

/// What a converted argument writes into the array the extension reads. The
/// union member is not read back here, because a test may not say `unsafe`
/// (D-U1).
#[test]
fn a_descriptor_carries_the_stripped_code_and_the_flags() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let declared = OPTIONAL_ARGUMENT | code::CSTRING;
    let converted = to_native(&mut cx, declared, Some(subject), 1).expect("a string converts");
    let filled = descriptor(declared, converted);
    assert_eq!(
        filled.r#type,
        code::CSTRING,
        "the optional bit must not reach the descriptor"
    );
    assert_eq!(filled.flags, ARGUMENT_EXISTS);

    let plain = to_native(&mut cx, code::CSTRING, Some(subject), 1).expect("a string converts");
    assert_eq!(descriptor(code::CSTRING, plain).r#type, code::CSTRING);
}
