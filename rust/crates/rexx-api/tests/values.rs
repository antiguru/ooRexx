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
    ARGUMENT_EXISTS, CStringPool, Class, Constants, Conversion, Converted, Failure, Host, Numeric,
    OPTIONAL_ARGUMENT, Raised, Repr, ResultRead, SPECIAL_ARGUMENT, Value, code, consumes_argument,
    descriptor, from_native, pointer_string, repr, result_read, rows, takes_argument_list,
    to_native,
};
use rexx_core::{BehaviourHandle, Body, Bytes, Heap, ObjRef, RootSet};

/// Stands in for the interpreter the table calls back into.
///
/// `string_value` models `requiredString`: a text object is its own string
/// value, a small integer has one that must be built, and anything in
/// `speechless` has none. The numeric readers parse a text object's bytes as
/// Rust parses a number, standing in for the interpreter's number syntax.
struct Interpreter {
    heap: Heap,
    method: bool,
    cself: Option<POINTER>,
    speechless: Vec<ObjRef>,
    /// Objects whose conversion raises a condition.
    raising: Vec<ObjRef>,
    variables: Vec<(Vec<u8>, ObjRef)>,
    locals: Table,
    receiver: ObjRef,
    scope: ObjRef,
    super_scope: ObjRef,
    argument_list: ObjRef,
    name: Vec<u8>,
    /// The calling activation's stems, by name with the trailing period.
    stems: Vec<(Vec<u8>, ObjRef)>,
    /// The objects `is_instance_of` answers true for, with their class.
    instances: Vec<(ObjRef, Class)>,
    /// What `logical` reports it tested, where it is not the argument.
    converted: Vec<ObjRef>,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            heap: Heap::new(),
            method: true,
            cself: None,
            speechless: Vec::new(),
            raising: Vec::new(),
            variables: Vec::new(),
            locals: Table::new(),
            receiver: ObjRef::NIL,
            scope: ObjRef::NIL,
            super_scope: ObjRef::NIL,
            argument_list: ObjRef::NIL,
            name: Vec::new(),
            stems: Vec::new(),
            instances: Vec::new(),
            converted: Vec::new(),
        }
    }

    fn text(&mut self, bytes: &[u8]) -> ObjRef {
        self.heap.alloc(Body::Text {
            bytes: Bytes::from_slice(bytes),
            num: None,
        })
    }

    fn array(&mut self) -> ObjRef {
        self.heap.alloc(Body::Array {
            dimensions: None,
            slots: vec![None],
        })
    }

    fn stem(&mut self, name: &[u8]) -> ObjRef {
        self.heap.alloc(Body::Stem {
            name: name.into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        })
    }

    /// A text object's bytes, parsed.
    fn parsed<T: std::str::FromStr>(&self, object: ObjRef) -> Result<Option<T>, Raised> {
        if self.raising.contains(&object) {
            return Err(Raised);
        }
        let text = self.string_bytes(object).map(Cow::into_owned);
        Ok(text.and_then(|bytes| String::from_utf8(bytes).ok()?.trim().parse().ok()))
    }

    /// What an answered object reads as: a small integer's digits, a text
    /// object's bytes.
    fn rendered(&self, object: ObjRef) -> Vec<u8> {
        match object.decode() {
            rexx_core::Decoded::SmallInt(number) => number.to_string().into_bytes(),
            _ => self
                .string_bytes(object)
                .expect("an answered number or string")
                .into_owned(),
        }
    }
}

impl Host for Interpreter {
    fn is_method(&self) -> bool {
        self.method
    }

    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        if self.raising.contains(&object) {
            return Err(Raised);
        }
        if self.speechless.contains(&object) {
            return Ok(None);
        }
        Ok(match object.decode() {
            rexx_core::Decoded::SmallInt(number) => Some(self.text(number.to_string().as_bytes())),
            rexx_core::Decoded::Text(_) => Some(object),
            _ => match self.heap.get(object).map(|found| &found.body) {
                Some(Body::Text { .. }) => Some(object),
                _ => None,
            },
        })
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

    fn numeric(&self) -> Numeric {
        unreachable!("no conversion reads the call context")
    }

    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Raised> {
        self.parsed(object)
    }

    fn signed_integer(
        &mut self,
        object: ObjRef,
        min: i64,
        max: i64,
    ) -> Result<Option<i64>, Raised> {
        Ok(self
            .parsed(object)?
            .filter(|number| (min..=max).contains(number)))
    }

    fn unsigned_integer(&mut self, object: ObjRef, max: u64) -> Result<Option<u64>, Raised> {
        Ok(self.parsed(object)?.filter(|number| *number <= max))
    }

    /// Tests the text `object` holds, and answers the first object in
    /// `converted` where it is not logical, standing in for the string the
    /// conversion made.
    fn logical(&mut self, object: ObjRef) -> Result<Result<bool, ObjRef>, Raised> {
        Ok(match self.parsed::<String>(object)?.as_deref() {
            Some("0") => Ok(false),
            Some("1") => Ok(true),
            _ => Err(self.converted.first().copied().unwrap_or(object)),
        })
    }

    fn array_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        if self.raising.contains(&object) {
            return Err(Raised);
        }
        Ok(match self.heap.get(object).map(|found| &found.body) {
            Some(Body::Array { .. }) => Some(object),
            _ => None,
        })
    }

    fn is_stem(&self, object: ObjRef) -> bool {
        matches!(
            self.heap.get(object).map(|found| &found.body),
            Some(Body::Stem { .. })
        )
    }

    fn context_stem(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        let Some(mut name) = self.parsed::<String>(object)? else {
            return Ok(None);
        };
        name.make_ascii_uppercase();
        if !name.ends_with('.') {
            name.push('.');
        }
        Ok(self
            .stems
            .iter()
            .find(|(held, _)| *held == name.as_bytes())
            .map(|(_, stem)| *stem))
    }

    fn is_instance_of(&mut self, object: ObjRef, class: Class) -> bool {
        self.instances.contains(&(object, class))
    }

    fn pointer_value(&self, object: ObjRef) -> Option<POINTER> {
        match &self.heap.get(object)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.pointer(),
            _ => None,
        }
    }

    fn string_value_text(&mut self, object: ObjRef) -> Vec<u8> {
        self.string_bytes(object)
            .map_or_else(|| b"an Object".to_vec(), Cow::into_owned)
    }

    fn receiver(&mut self) -> ObjRef {
        self.receiver
    }

    fn scope(&mut self) -> ObjRef {
        self.scope
    }

    fn super_scope(&mut self) -> ObjRef {
        self.super_scope
    }

    fn arguments(&mut self) -> ObjRef {
        self.argument_list
    }

    fn message_name(&mut self) -> Vec<u8> {
        self.name.clone()
    }

    fn unsigned_number(&mut self, value: u64) -> ObjRef {
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_string(&mut self, bytes: &[u8]) -> ObjRef {
        self.text(bytes)
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        self.text(format!("{value} at {precision}").as_bytes())
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
        .filter(|code| !consumes_argument(*code))
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
    // A code the table does not know is `processArguments`' `default:`,
    // which consumes an argument.
    assert!(consumes_argument(9999));
}

/// A code the header does not define is a signature error where the oracle
/// reaches a `default:` with it, and a missing argument first where the
/// argument is absent and the code not optional. Measured through a forged
/// extension, oracle: code 9 with no argument is 88.901, with one 93.968, and
/// optional with none 93.968.
#[test]
fn a_code_the_table_does_not_know_is_a_signature_error() {
    let mut host = Interpreter::new();
    let supplied = host.text(b"supplied");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, 9999, None, 1),
        Err(Failure::MissingArgument { position: 1 }),
        "a required argument is checked before the code"
    );
    assert_eq!(
        to_native(&mut cx, 9999, Some(supplied), 1),
        Err(Failure::Signature)
    );
    assert_eq!(
        to_native(&mut cx, OPTIONAL_ARGUMENT | 9999, None, 1),
        Err(Failure::Signature)
    );
    assert_eq!(
        from_native(&mut cx, 9999, Value::Int(0)),
        Err(Failure::ResultSignature)
    );
}

/// A result word carrying the optional bit is no row of `valueToObject`'s
/// switch, which reads the word unstripped: measured through a forged
/// extension, oracle 93.968 where the stripped word would convert.
#[test]
fn a_result_word_carrying_the_optional_bit_is_a_signature_error() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, OPTIONAL_ARGUMENT | code::INT, Value::Int(7)),
        Err(Failure::ResultSignature)
    );
    assert_eq!(
        from_native(&mut cx, code::INT, Value::Int(7)),
        Ok(Some(ObjRef::small_int(7).expect("a small integer")))
    );
}

/// A signature error's number differs by context.
#[test]
fn the_signature_error_is_93_968_in_a_method_and_40_918_in_a_call() {
    assert_eq!(Failure::Signature.error_number(true), Some(93968));
    assert_eq!(Failure::Signature.error_number(false), Some(40918));
    assert_eq!(Failure::ResultSignature.error_number(true), Some(93968));
    assert_eq!(Failure::ResultSignature.error_number(false), Some(40918));
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

/// A condition raised while an argument converts is the host's to raise, not
/// 88.909: measured, oracle, a `MAKESTRING` doing `raise syntax 40.1` for a
/// `CSTRING` or a `RexxStringObject` parameter is `Error 40.1` at rc 216.
#[test]
fn a_raise_inside_the_string_conversion_is_not_a_missing_string_value() {
    for declared in [code::CSTRING, code::REXX_STRING_OBJECT] {
        let mut host = Interpreter::new();
        let subject = host.text(b"a string long enough to reach the heap");
        host.raising.push(subject);
        let mut strings = CStringPool::new();
        let mut cx = Conversion {
            host: &mut host,
            strings: &mut strings,
        };
        let refusal = to_native(&mut cx, declared, Some(subject), 1)
            .expect_err("a conversion that raised cannot answer a value");
        assert_eq!(refusal, Failure::Raised, "code {declared}");
        assert_eq!(refusal.error_number(true), None);
    }
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

// ---------------------------------------------------------------- the double

#[test]
fn the_double_row_converts_what_the_host_reads() {
    let mut host = Interpreter::new();
    let subject = host.text(b"2.25");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, code::DOUBLE, Some(subject), 1).expect("2.25 converts");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    assert_eq!(converted.value, Value::Double(2.25));
    assert_eq!(descriptor(code::DOUBLE, converted).r#type, code::DOUBLE);
}

/// Measured against the oracle: `RxCalcSqrt('abc')` is 88.921 naming argument
/// one and the argument itself.
#[test]
fn a_double_the_host_cannot_read_is_88_921_naming_the_argument() {
    let mut host = Interpreter::new();
    let subject = host.text(b"abc");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let refused = to_native(&mut cx, code::DOUBLE, Some(subject), 2).expect_err("abc is no double");
    assert_eq!(
        refused,
        Failure::InvalidDouble {
            position: 2,
            argument: subject,
        }
    );
    assert_eq!(refused.error_number(false), Some(88921));
}

#[test]
fn a_raise_reading_a_double_is_the_hosts_condition() {
    let mut host = Interpreter::new();
    let subject = host.text(b"2.25");
    host.raising.push(subject);
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, code::DOUBLE, Some(subject), 1),
        Err(Failure::Raised)
    );
}

/// Measured against the oracle: `RxCalcSqrt(, 2)` is 88.901, and the
/// optional form writes a zero with no flags.
#[test]
fn an_absent_double_is_88_901_or_a_zero() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        to_native(&mut cx, code::DOUBLE, None, 1),
        Err(Failure::MissingArgument { position: 1 })
    );
    let omitted = to_native(&mut cx, OPTIONAL_ARGUMENT | code::DOUBLE, None, 1)
        .expect("an optional double may be left out");
    assert_eq!(omitted.value, Value::Omitted);
    assert_eq!(omitted.flags, 0);
}

// ------------------------------------------------ the positive whole number

#[test]
fn the_positive_whole_number_row_converts_what_the_host_reads() {
    let mut host = Interpreter::new();
    let subject = host.text(b"16");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted =
        to_native(&mut cx, code::POSITIVE_WHOLENUMBER_T, Some(subject), 2).expect("16 converts");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    assert_eq!(converted.value, Value::Isize(16));
}

/// Measured against the oracle: a precision of `0` to `RxCalcSqrt` is 88.905
/// naming argument two and the argument itself.
#[test]
fn a_whole_number_below_one_is_88_905_naming_the_argument() {
    let mut host = Interpreter::new();
    let subject = host.text(b"0");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let refused = to_native(&mut cx, code::POSITIVE_WHOLENUMBER_T, Some(subject), 2)
        .expect_err("zero is not positive");
    assert_eq!(
        refused,
        Failure::NotPositive {
            position: 2,
            argument: subject,
        }
    );
    assert_eq!(refused.error_number(false), Some(88905));
}

#[test]
fn an_omitted_optional_whole_number_is_a_zero_with_no_flags() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let omitted = to_native(
        &mut cx,
        OPTIONAL_ARGUMENT | code::POSITIVE_WHOLENUMBER_T,
        None,
        2,
    )
    .expect("an optional whole number may be left out");
    assert_eq!(omitted.value, Value::Omitted);
    assert_eq!(omitted.flags, 0);
}

// ------------------------------------------------------- the RexxObjectPtr

#[test]
fn a_returned_object_converts_back_through_its_handle() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let handle = host.locals().register(subject);
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, code::REXX_OBJECT_PTR, Value::Object(handle)),
        Ok(Some(subject))
    );
    assert_eq!(
        from_native(
            &mut cx,
            code::REXX_OBJECT_PTR,
            Value::Object(std::ptr::null_mut())
        ),
        Ok(None)
    );
}

#[test]
fn a_returned_object_the_table_dropped_does_not_convert_back() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let handle = host.locals().register(subject);
    host.locals().clear();
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    assert_eq!(
        from_native(&mut cx, code::REXX_OBJECT_PTR, Value::Object(handle)),
        Err(Failure::StaleHandle)
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
        Err(Failure::ResultSignature)
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

// ------------------------------------------------------- every row, as a set

/// A value of `repr`'s shape, converted back through `code`'s row.
fn back(host: &mut Interpreter, code: u16, value: Value) -> Result<Option<ObjRef>, Failure> {
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host,
        strings: &mut strings,
    };
    from_native(&mut cx, code, value)
}

/// A value of each member's shape that every row of that member accepts.
fn sample(repr: Repr) -> Value {
    match repr {
        Repr::Object => Value::Object(std::ptr::null_mut()),
        Repr::CString => Value::CString(std::ptr::null()),
        Repr::Pointer => Value::Pointer(std::ptr::null_mut()),
        Repr::Int => Value::Int(1),
        Repr::Int8 => Value::Int8(1),
        Repr::Int16 => Value::Int16(1),
        Repr::Int32 => Value::Int32(1),
        Repr::Int64 => Value::Int64(1),
        Repr::Uint8 => Value::Uint8(1),
        Repr::Uint16 => Value::Uint16(1),
        Repr::Uint32 => Value::Uint32(1),
        Repr::Uint64 => Value::Uint64(1),
        Repr::Isize => Value::Isize(1),
        Repr::Usize => Value::Usize(1),
        Repr::Double => Value::Double(1.0),
        Repr::Float => Value::Float(1.0),
    }
}

/// The rows whose result `valueToObject` refuses (`NativeActivation.cpp:855`)
/// are exactly the special ones, and every other row converts a value of its
/// own member back. Measured through a forged extension, oracle: every
/// special code as a routine's result type is 40.918, and `ARGLIST`, `NAME`
/// and `CSELF` as a method's are 93.968.
#[test]
fn the_result_of_exactly_the_special_rows_is_refused() {
    let mut host = Interpreter::new();
    let mut refused = Vec::new();
    for (code, name) in rows() {
        let member = repr(code).expect("every row names a member");
        match back(&mut host, code, sample(member)) {
            Err(Failure::ResultSignature) => refused.push(code),
            Ok(_) => {}
            other => panic!("REXX_VALUE_{name} answered {other:?}"),
        }
        assert_eq!(
            result_read(code).is_none(),
            refused.last() == Some(&code),
            "REXX_VALUE_{name}'s result is read though it is refused, or the other way"
        );
    }
    assert_eq!(
        refused,
        vec![
            code::ARGLIST,
            code::NAME,
            code::SCOPE,
            code::CSELF,
            code::OSELF,
            code::SUPER
        ]
    );
}

/// A result is read as the member its row names, except a `CSTRING`, whose
/// bytes are what comes back.
#[test]
fn a_result_is_read_as_its_rows_member_and_a_cstring_as_its_bytes() {
    assert_eq!(result_read(code::CSTRING), Some(ResultRead::Text));
    assert_eq!(
        result_read(code::UINT16_T),
        Some(ResultRead::Member(Repr::Uint16))
    );
    assert_eq!(result_read(OPTIONAL_ARGUMENT | code::INT), None);
    assert_eq!(result_read(9999), None);
}

/// The argument list waives the check for arguments nothing consumes, and it
/// alone does: measured, oracle, `TestArglistArg(1, 2, 3)` answers where
/// `TestNameArg(1)` is 88.922.
#[test]
fn only_the_argument_list_lifts_the_too_many_check() {
    let lifting: Vec<u16> = rows()
        .map(|(code, _)| code)
        .filter(|code| takes_argument_list(*code))
        .collect();
    assert_eq!(lifting, vec![code::ARGLIST]);
    assert!(takes_argument_list(OPTIONAL_ARGUMENT | code::ARGLIST));
    assert!(!takes_argument_list(9999));
}

#[test]
fn every_new_refusal_carries_the_number_the_oracle_raises() {
    let argument = ObjRef::NIL;
    for (failure, method, routine) in [
        (
            Failure::NotNonnegative {
                position: 1,
                argument,
            },
            88904,
            88904,
        ),
        (
            Failure::OutOfRange {
                position: 1,
                min: 0,
                max: 1,
                argument,
            },
            88907,
            88907,
        ),
        (Failure::NotLogical { found: argument }, 34901, 34901),
        (Failure::NotArray { argument }, 98913, 98913),
        (
            Failure::NotInstance {
                position: 1,
                class: Class::Class,
            },
            88914,
            88914,
        ),
        (
            Failure::NotPointerString {
                position: 1,
                argument,
            },
            88919,
            88919,
        ),
        (
            Failure::NoStem {
                position: 1,
                argument,
            },
            93969,
            40919,
        ),
    ] {
        assert_eq!(failure.error_number(true), Some(method), "{failure:?}");
        assert_eq!(failure.error_number(false), Some(routine), "{failure:?}");
    }
    assert_eq!(Class::MutableBuffer.id(), "MutableBuffer");
    assert_eq!(Class::VariableReference.id(), "VariableReference");
    assert_eq!(Class::Pointer.id(), "Pointer");
}

// -------------------------------------------------------- the special rows

/// Runs `declared` through `to_native` with no argument, as a special row is
/// run.
fn special(host: &mut Interpreter, declared: u16) -> (Result<Converted, Failure>, CStringPool) {
    let mut strings = CStringPool::new();
    let converted = {
        let mut cx = Conversion {
            host,
            strings: &mut strings,
        };
        to_native(&mut cx, declared, None, 1)
    };
    (converted, strings)
}

#[test]
fn the_arglist_row_hands_over_the_calls_argument_array_in_a_method_and_a_routine() {
    for method in [true, false] {
        let mut host = Interpreter::new();
        host.method = method;
        host.argument_list = host.array();
        let (converted, _) = special(&mut host, code::ARGLIST);
        let converted = converted.expect("ARGLIST needs no argument");
        assert_eq!(converted.flags, ARGUMENT_EXISTS | SPECIAL_ARGUMENT);
        let Value::Object(handle) = converted.value else {
            panic!("ARGLIST answers a handle, got {:?}", converted.value)
        };
        assert_eq!(host.locals().resolve(handle), Some(host.argument_list));
    }
    let mut host = Interpreter::new();
    assert_eq!(
        back(
            &mut host,
            code::ARGLIST,
            Value::Object(std::ptr::null_mut())
        ),
        Err(Failure::ResultSignature)
    );
}

#[test]
fn the_name_row_points_at_the_name_the_call_was_made_by() {
    let mut host = Interpreter::new();
    host.method = false;
    host.name = b"TESTNAMEARG".to_vec();
    let (converted, strings) = special(&mut host, code::NAME);
    let converted = converted.expect("NAME needs no argument");
    assert_eq!(converted.flags, ARGUMENT_EXISTS | SPECIAL_ARGUMENT);
    let Value::CString(pointer) = converted.value else {
        panic!("NAME answers a CSTRING, got {:?}", converted.value)
    };
    assert_eq!(strings.bytes_at(pointer), Some(&b"TESTNAMEARG"[..]));
    assert_eq!(
        back(&mut host, code::NAME, Value::CString(std::ptr::null())),
        Err(Failure::ResultSignature)
    );
}

/// `OSELF`, `SCOPE` and `SUPER` hand a method the object the host names for
/// each, and are a signature error in a routine: measured through a forged
/// routine library, oracle 40.918 for each.
fn a_method_only_object_row(declared: u16, pick: fn(&mut Interpreter) -> ObjRef) {
    let mut host = Interpreter::new();
    host.receiver = host.text(b"the receiver");
    host.scope = host.text(b"the scope");
    host.super_scope = host.text(b"the scope above");
    let (converted, _) = special(&mut host, declared);
    let converted = converted.expect("a method has one");
    assert_eq!(converted.flags, ARGUMENT_EXISTS | SPECIAL_ARGUMENT);
    let Value::Object(handle) = converted.value else {
        panic!(
            "code {declared} answers a handle, got {:?}",
            converted.value
        )
    };
    let expected = pick(&mut host);
    assert_eq!(host.locals().resolve(handle), Some(expected));

    host.method = false;
    let (refused, _) = special(&mut host, declared);
    assert_eq!(refused, Err(Failure::Signature));
    assert_eq!(
        back(&mut host, declared, Value::Object(std::ptr::null_mut())),
        Err(Failure::ResultSignature)
    );
}

#[test]
fn the_oself_row_hands_a_method_its_receiver() {
    a_method_only_object_row(code::OSELF, |host| host.receiver);
}

#[test]
fn the_scope_row_hands_a_method_its_scope() {
    a_method_only_object_row(code::SCOPE, |host| host.scope);
}

#[test]
fn the_super_row_hands_a_method_the_scope_above_its_own() {
    a_method_only_object_row(code::SUPER, |host| host.super_scope);
}

#[test]
fn a_cself_is_never_converted_back() {
    let mut host = Interpreter::new();
    assert_eq!(
        back(&mut host, code::CSELF, Value::Pointer(std::ptr::null_mut())),
        Err(Failure::ResultSignature)
    );
}

// ---------------------------------------------------------- the integer rows

/// `text` converted as `declared` at position 2.
fn convert(host: &mut Interpreter, declared: u16, text: &str) -> Result<Converted, Failure> {
    let argument = host.text(text.as_bytes());
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host,
        strings: &mut strings,
    };
    to_native(&mut cx, declared, Some(argument), 2)
}

/// A signed integer row converts both ends of its range into its own member,
/// refuses one past each naming that range, and converts a value back to its
/// digits. The ranges are the ones 88.907 names, measured through
/// `orxmethod`'s echo methods.
fn signed_row(declared: u16, min: i128, max: i128, member: fn(i128) -> Value) {
    let mut host = Interpreter::new();
    for end in [min, max] {
        let converted = convert(&mut host, declared, &end.to_string()).expect("an end converts");
        assert_eq!(converted.value, member(end), "code {declared} at {end}");
        assert_eq!(converted.flags, ARGUMENT_EXISTS);
        let object = back(&mut host, declared, converted.value)
            .expect("a value converts back")
            .expect("to an object");
        assert_eq!(host.rendered(object), end.to_string().into_bytes());
    }
    for past in [min - 1, max + 1] {
        let refused = convert(&mut host, declared, &past.to_string())
            .expect_err("one past an end is out of range");
        let Failure::OutOfRange {
            position,
            min: named_min,
            max: named_max,
            ..
        } = refused
        else {
            panic!("code {declared} at {past} refused with {refused:?}")
        };
        assert_eq!((position, named_min, named_max), (2, min, max));
    }
}

/// [`signed_row`] for an unsigned row, whose range starts at zero.
fn unsigned_row(declared: u16, max: i128, member: fn(i128) -> Value) {
    signed_row(declared, 0, max, member);
}

/// The value an in-range `i128` is, as a member of type `T`.
fn narrow<T: TryFrom<i128>>(value: i128) -> T {
    T::try_from(value).unwrap_or_else(|_| panic!("{value} is in range"))
}

#[test]
fn the_int_row_converts_its_range_both_ways() {
    signed_row(code::INT, i128::from(i32::MIN), i128::from(i32::MAX), |v| {
        Value::Int(narrow(v))
    });
}

#[test]
fn the_int8_row_converts_its_range_both_ways() {
    signed_row(code::INT8_T, -128, 127, |v| Value::Int8(narrow(v)));
}

#[test]
fn the_int16_row_converts_its_range_both_ways() {
    signed_row(code::INT16_T, -32768, 32767, |v| Value::Int16(narrow(v)));
}

#[test]
fn the_int32_row_converts_its_range_both_ways() {
    signed_row(
        code::INT32_T,
        i128::from(i32::MIN),
        i128::from(i32::MAX),
        |v| Value::Int32(narrow(v)),
    );
}

#[test]
fn the_int64_row_converts_its_range_both_ways() {
    signed_row(
        code::INT64_T,
        i128::from(i64::MIN),
        i128::from(i64::MAX),
        |v| Value::Int64(narrow(v)),
    );
}

#[test]
fn the_intptr_row_converts_its_range_both_ways() {
    signed_row(
        code::INTPTR_T,
        i128::from(i64::MIN),
        i128::from(i64::MAX),
        |v| Value::Isize(narrow(v)),
    );
}

#[test]
fn the_ssize_row_converts_its_range_both_ways() {
    signed_row(
        code::SSIZE_T,
        i128::from(i64::MIN),
        i128::from(i64::MAX),
        |v| Value::Isize(narrow(v)),
    );
}

#[test]
fn the_wholenumber_row_converts_its_range_both_ways() {
    signed_row(
        code::WHOLENUMBER_T,
        -999_999_999_999_999_999,
        999_999_999_999_999_999,
        |v| Value::Isize(narrow(v)),
    );
}

#[test]
fn the_uint8_row_converts_its_range_both_ways() {
    unsigned_row(code::UINT8_T, 255, |v| Value::Uint8(narrow(v)));
}

#[test]
fn the_uint16_row_converts_its_range_both_ways() {
    unsigned_row(code::UINT16_T, 65535, |v| Value::Uint16(narrow(v)));
}

#[test]
fn the_uint32_row_converts_its_range_both_ways() {
    unsigned_row(code::UINT32_T, i128::from(u32::MAX), |v| {
        Value::Uint32(narrow(v))
    });
}

#[test]
fn the_uint64_row_converts_its_range_both_ways() {
    unsigned_row(code::UINT64_T, i128::from(u64::MAX), |v| {
        Value::Uint64(narrow(v))
    });
}

#[test]
fn the_uintptr_row_converts_its_range_both_ways() {
    unsigned_row(code::UINTPTR_T, i128::from(u64::MAX), |v| {
        Value::Usize(narrow(v))
    });
}

#[test]
fn the_size_row_converts_its_range_both_ways() {
    unsigned_row(code::SIZE_T, i128::from(u64::MAX), |v| {
        Value::Usize(narrow(v))
    });
}

#[test]
fn the_stringsize_row_converts_its_range_both_ways() {
    unsigned_row(code::STRINGSIZE_T, 999_999_999_999_999_999, |v| {
        Value::Usize(narrow(v))
    });
}

/// Measured, oracle: `TestNonnegativeWholeNumberArg(-1)` is 88.904 and `0`
/// converts.
#[test]
fn the_nonnegative_whole_number_row_converts_zero_up_both_ways() {
    let mut host = Interpreter::new();
    let converted = convert(&mut host, code::NONNEGATIVE_WHOLENUMBER_T, "0").expect("zero");
    assert_eq!(converted.value, Value::Isize(0));
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let object = back(&mut host, code::NONNEGATIVE_WHOLENUMBER_T, Value::Isize(7))
        .expect("converts back")
        .expect("to an object");
    assert_eq!(host.rendered(object), b"7");
    let refused =
        convert(&mut host, code::NONNEGATIVE_WHOLENUMBER_T, "-1").expect_err("below zero");
    assert!(
        matches!(refused, Failure::NotNonnegative { position: 2, .. }),
        "{refused:?}"
    );
    assert!(
        convert(
            &mut host,
            code::NONNEGATIVE_WHOLENUMBER_T,
            "1000000000000000000"
        )
        .is_err()
    );
}

#[test]
fn a_positive_whole_number_converts_back_to_its_digits() {
    let mut host = Interpreter::new();
    let object = back(&mut host, code::POSITIVE_WHOLENUMBER_T, Value::Isize(16))
        .expect("converts back")
        .expect("to an object");
    assert_eq!(host.rendered(object), b"16");
}

#[test]
fn a_raise_reading_an_integer_is_the_hosts_condition() {
    for declared in [code::INT, code::UINT64_T, code::NONNEGATIVE_WHOLENUMBER_T] {
        let mut host = Interpreter::new();
        let subject = host.text(b"7");
        host.raising.push(subject);
        let mut strings = CStringPool::new();
        let mut cx = Conversion {
            host: &mut host,
            strings: &mut strings,
        };
        assert_eq!(
            to_native(&mut cx, declared, Some(subject), 1),
            Err(Failure::Raised),
            "code {declared}"
        );
    }
}

// ------------------------------------------------------------ the logical_t

/// Measured, oracle: `TestLogicalArg(3)` is 34.901, and any value but zero
/// an extension returns is `1`. The refusal names the string that was tested,
/// which for an object converted through `MAKESTRING` or `STRING` is not the
/// argument: measured, oracle, `.array~of(1,2)` is `found "1<LF>2"`.
#[test]
fn the_logical_row_converts_zero_and_one_and_any_non_zero_back_as_one() {
    let mut host = Interpreter::new();
    for (text, truth) in [("0", 0), ("1", 1)] {
        let converted = convert(&mut host, code::LOGICAL_T, text).expect("a logical");
        assert_eq!(converted.value, Value::Usize(truth));
    }
    let tested = host.text(b"the string the conversion made");
    host.converted.push(tested);
    let refused = convert(&mut host, code::LOGICAL_T, "2").expect_err("not logical");
    assert_eq!(refused, Failure::NotLogical { found: tested });
    for (written, answer) in [(0, &b"0"[..]), (1, b"1"), (5, b"1"), (usize::MAX, b"1")] {
        let object = back(&mut host, code::LOGICAL_T, Value::Usize(written))
            .expect("converts back")
            .expect("to an object");
        assert_eq!(host.rendered(object), answer, "written {written}");
    }
}

// ------------------------------------------------------ the double and float

#[test]
fn a_double_result_is_rendered_at_nine_digits() {
    let mut host = Interpreter::new();
    let object = back(&mut host, code::DOUBLE, Value::Double(2.25))
        .expect("converts back")
        .expect("to an object");
    assert_eq!(host.rendered(object), b"2.25 at 9");
}

/// Measured, oracle: `TestFloatArg('zz')` is 88.921 as a double's is, and a
/// float result renders as the double it widens to, at nine digits.
#[test]
fn the_float_row_narrows_a_double_and_widens_it_back() {
    let mut host = Interpreter::new();
    let converted = convert(&mut host, code::FLOAT, "1.5").expect("1.5 converts");
    assert_eq!(converted.value, Value::Float(1.5));
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let refused = convert(&mut host, code::FLOAT, "zz").expect_err("no double");
    assert!(
        matches!(refused, Failure::InvalidDouble { position: 2, .. }),
        "{refused:?}"
    );
    let object = back(&mut host, code::FLOAT, Value::Float(0.5))
        .expect("converts back")
        .expect("to an object");
    assert_eq!(host.rendered(object), b"0.5 at 9");
}

// ------------------------------------------------------- the CSTRING result

/// A `CSTRING` result is the pool's copy of what the extension answered, cut
/// at its first NUL; a null one is no object, and a pointer the pool did not
/// mint is refused rather than read. Measured, oracle: a routine answering
/// `"ab\0cd"` has length 2 and one answering `NULL` is 44.1.
#[test]
fn a_cstring_result_is_the_pools_bytes_up_to_the_first_nul() {
    let mut host = Interpreter::new();
    let mut strings = CStringPool::new();
    let whole = strings.intern(b"hello");
    let cut = strings.intern(b"ab\0cd");
    let foreign = std::ptr::without_provenance::<std::ffi::c_char>(0x5eed);
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let answered = from_native(&mut cx, code::CSTRING, Value::CString(whole))
        .expect("converts back")
        .expect("to an object");
    let cut_answer = from_native(&mut cx, code::CSTRING, Value::CString(cut))
        .expect("converts back")
        .expect("to an object");
    assert_eq!(
        from_native(&mut cx, code::CSTRING, Value::CString(std::ptr::null())),
        Ok(None)
    );
    assert_eq!(
        from_native(&mut cx, code::CSTRING, Value::CString(foreign)),
        Err(Failure::StaleHandle)
    );
    assert_eq!(
        from_native(&mut cx, code::CSTRING, Value::Omitted),
        Err(Failure::ResultSignature)
    );
    assert_eq!(host.rendered(answered), b"hello");
    assert_eq!(host.rendered(cut_answer), b"ab");
}

// ----------------------------------------------------------- the object rows

#[test]
fn the_object_row_hands_over_the_argument_itself() {
    let mut host = Interpreter::new();
    let subject = host.text(b"a string long enough to reach the heap");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted =
        to_native(&mut cx, code::REXX_OBJECT_PTR, Some(subject), 1).expect("anything converts");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let Value::Object(handle) = converted.value else {
        panic!("an object row answers a handle, got {:?}", converted.value)
    };
    assert_eq!(cx.host.locals().resolve(handle), Some(subject));
}

/// Measured, oracle: `TestArrayArg(.object~new)` is 98.913 and a string
/// converts through its `makeArray`.
#[test]
fn the_array_row_hands_over_what_the_host_converts_and_refuses_the_rest() {
    let mut host = Interpreter::new();
    let array = host.array();
    let refused_object = host.text(b"no array value");
    let raising = host.array();
    host.raising.push(raising);
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, code::REXX_ARRAY_OBJECT, Some(array), 1).expect("converts");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let Value::Object(handle) = converted.value else {
        panic!("an array row answers a handle, got {:?}", converted.value)
    };
    assert_eq!(cx.host.locals().resolve(handle), Some(array));
    assert_eq!(
        from_native(&mut cx, code::REXX_ARRAY_OBJECT, converted.value),
        Ok(Some(array))
    );
    assert_eq!(
        to_native(&mut cx, code::REXX_ARRAY_OBJECT, Some(refused_object), 1),
        Err(Failure::NotArray {
            argument: refused_object
        })
    );
    assert_eq!(
        to_native(&mut cx, code::REXX_ARRAY_OBJECT, Some(raising), 1),
        Err(Failure::Raised)
    );
}

/// A stem converts in a method and a call alike; a stem's name only in a
/// call. Measured, oracle: `TestStemArg('zz')` is 93.969 from `orxmethod` and
/// the caller's stem from `orxfunction`, whose `TestStemArg('a.b')` is 40.919.
#[test]
fn the_stem_row_takes_a_stem_anywhere_and_a_name_only_in_a_call() {
    let mut host = Interpreter::new();
    let stem = host.stem(b"X.");
    let held = host.stem(b"ZZ.");
    host.stems.push((b"ZZ.".to_vec(), held));
    let name = host.text(b"zz");
    let unknown = host.text(b"a.b");
    let mut strings = CStringPool::new();
    let handed = |cx: &mut Conversion<'_>, converted: Converted| match converted.value {
        Value::Object(handle) => cx.host.locals().resolve(handle),
        other => panic!("a stem row answers a handle, got {other:?}"),
    };
    {
        let mut cx = Conversion {
            host: &mut host,
            strings: &mut strings,
        };
        let converted = to_native(&mut cx, code::REXX_STEM_OBJECT, Some(stem), 1).expect("a stem");
        assert_eq!(handed(&mut cx, converted), Some(stem));
        assert_eq!(
            to_native(&mut cx, code::REXX_STEM_OBJECT, Some(name), 1),
            Err(Failure::NoStem {
                position: 1,
                argument: name
            }),
            "a method has no caller's variables to name"
        );
        let handle = cx.host.locals().register(stem);
        assert_eq!(
            from_native(&mut cx, code::REXX_STEM_OBJECT, Value::Object(handle)),
            Ok(Some(stem))
        );
    }

    host.method = false;
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, code::REXX_STEM_OBJECT, Some(name), 1).expect("a name");
    assert_eq!(handed(&mut cx, converted), Some(held));
    let converted = to_native(&mut cx, code::REXX_STEM_OBJECT, Some(stem), 1).expect("a stem");
    assert_eq!(handed(&mut cx, converted), Some(stem));
    assert_eq!(
        to_native(&mut cx, code::REXX_STEM_OBJECT, Some(unknown), 3),
        Err(Failure::NoStem {
            position: 3,
            argument: unknown
        })
    );
}

/// An argument whose row requires an instance of `class` is handed over when
/// the host says it is one, and refused naming the class otherwise.
/// Measured, oracle, 88.914 for each class.
fn an_instance_row(declared: u16, class: Class) {
    let mut host = Interpreter::new();
    let instance = host.text(b"an instance");
    let other = host.text(b"something else");
    host.instances.push((instance, class));
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, declared, Some(instance), 1).expect("an instance");
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let Value::Object(handle) = converted.value else {
        panic!(
            "code {declared} answers a handle, got {:?}",
            converted.value
        )
    };
    assert_eq!(cx.host.locals().resolve(handle), Some(instance));
    assert_eq!(
        from_native(&mut cx, declared, converted.value),
        Ok(Some(instance))
    );
    assert_eq!(
        to_native(&mut cx, declared, Some(other), 2),
        Err(Failure::NotInstance { position: 2, class })
    );
}

#[test]
fn the_class_row_takes_only_a_class() {
    an_instance_row(code::REXX_CLASS_OBJECT, Class::Class);
}

#[test]
fn the_mutable_buffer_row_takes_only_a_mutable_buffer() {
    an_instance_row(code::REXX_MUTABLE_BUFFER_OBJECT, Class::MutableBuffer);
}

#[test]
fn the_variable_reference_row_takes_only_a_variable_reference() {
    an_instance_row(
        code::REXX_VARIABLE_REFERENCE_OBJECT,
        Class::VariableReference,
    );
}

// --------------------------------------------------- the POINTER rows

/// Measured, oracle: `TestPointerArg('x')` is 88.914 naming the Pointer
/// class, and `TestPointerArg(TestPointerValue())` is `1`.
#[test]
fn the_pointer_row_unwraps_a_pointer_and_refuses_anything_else() {
    let mut host = Interpreter::new();
    let address = std::ptr::without_provenance_mut::<std::ffi::c_void>(0x5eed_0000);
    let pointer = host.new_pointer(address);
    let other = host.text(b"x");
    let mut strings = CStringPool::new();
    let mut cx = Conversion {
        host: &mut host,
        strings: &mut strings,
    };
    let converted = to_native(&mut cx, code::POINTER, Some(pointer), 1).expect("a pointer");
    assert_eq!(converted.value, Value::Pointer(address));
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    assert_eq!(
        to_native(&mut cx, code::POINTER, Some(other), 1),
        Err(Failure::NotInstance {
            position: 1,
            class: Class::Pointer
        })
    );
    let answered = from_native(&mut cx, code::POINTER, Value::Pointer(address))
        .expect("converts back")
        .expect("to an object");
    assert_eq!(cx.host.pointer_value(answered), Some(address));
}

#[test]
fn the_pointer_string_row_reads_an_address_and_writes_one_back() {
    let mut host = Interpreter::new();
    let converted = convert(&mut host, code::POINTERSTRING, "0x5eed").expect("an address");
    assert_eq!(
        converted.value,
        Value::Pointer(std::ptr::without_provenance_mut(0x5eed))
    );
    assert_eq!(converted.flags, ARGUMENT_EXISTS);
    let refused = convert(&mut host, code::POINTERSTRING, "zz").expect_err("no address");
    assert!(
        matches!(refused, Failure::NotPointerString { position: 2, .. }),
        "{refused:?}"
    );
    for (address, rendered) in [(0x5eed, &b"0x5eed"[..]), (0, b"0x0")] {
        let object = back(
            &mut host,
            code::POINTERSTRING,
            Value::Pointer(std::ptr::without_provenance_mut(address)),
        )
        .expect("converts back")
        .expect("to an object");
        assert_eq!(host.rendered(object), rendered);
    }
}

/// The forms measured on the oracle over the address `TestPointerStringValue`
/// answers, here over `0x7f5d14eeece0`, what `strtoul` reads past the end of
/// the range, and the `(nil)` spelling glibc prints for a null pointer and
/// reads back.
#[test]
fn a_pointer_string_is_read_as_sscanf_reads_0x_p() {
    let hex = "7f5d14eeece0";
    let address = 0x7f5d_14ee_ece0_usize;
    for (text, read) in [
        (format!("0x{hex}"), Some(address)),
        (format!("0x0x{hex}"), Some(address)),
        (format!("0x0X{hex}"), Some(address)),
        (format!("0x {hex}"), Some(address)),
        (format!("0x\t{hex}"), Some(address)),
        (format!("0x\n{hex}"), Some(address)),
        (format!("0x\u{b}{hex}"), Some(address)),
        (format!("0x{hex}zz"), Some(address)),
        (format!("0x{}", hex.to_uppercase()), Some(address)),
        (format!("0x+{hex}"), Some(address)),
        (format!("0x000{hex}"), Some(address)),
        (format!("0x-{:x}", address.wrapping_neg()), Some(address)),
        (format!("0x-0x{:x}", address.wrapping_neg()), Some(address)),
        ("0x0".to_string(), Some(0)),
        ("0x(nil)".to_string(), Some(0)),
        ("0x (nil)".to_string(), Some(0)),
        ("0x\t(nil)".to_string(), Some(0)),
        ("0x(NIL)".to_string(), Some(0)),
        ("0x(Nil)".to_string(), Some(0)),
        ("0x(nIL)".to_string(), Some(0)),
        ("0x(nil)zz".to_string(), Some(0)),
        ("0x(nil)(nil)".to_string(), Some(0)),
        ("0x0x(nil)".to_string(), Some(0)),
        ("0x0X(nil)".to_string(), Some(0)),
        ("0x-(nil)".to_string(), None),
        ("0x+(nil)".to_string(), None),
        ("0x-0x(nil)".to_string(), None),
        ("0x(nil".to_string(), None),
        ("0x(nill)".to_string(), None),
        ("0x(ni)".to_string(), None),
        ("0x( nil)".to_string(), None),
        ("0x(nil )".to_string(), None),
        ("0x0x (nil)".to_string(), None),
        (format!("0x1{}", "0".repeat(17)), Some(usize::MAX)),
        (format!("0x-1{}", "0".repeat(17)), Some(usize::MAX)),
        (format!("0X{hex}"), None),
        (format!(" 0x{hex}"), None),
        ("0xg".to_string(), None),
        ("0x0xg".to_string(), None),
        ("0x0x".to_string(), None),
        ("0x-".to_string(), None),
        ("0x ".to_string(), None),
        ("0x--1".to_string(), None),
        ("0x+-1".to_string(), None),
        (format!("0x0x {hex}"), None),
        (format!("0x\0{hex}"), None),
        ("12".to_string(), None),
        ("0x".to_string(), None),
    ] {
        assert_eq!(pointer_string(text.as_bytes()), read, "{text:?}");
    }
}
