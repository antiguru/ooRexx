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

//! `Object`'s own methods: identity, hashing, the operators, `objectName`,
//! `setMethod`, `copy`, `request`, and `run`/`send`/`start` with the `Message`
//! readers.

use crate::scheduler::{First, ParkReason, Scheduler, StartedSend};

use super::{
    Behaviour, BehaviourId, Body, Cleared, DEFAULTNAME, Failure, Interp, Loud, MESSAGE_ARGUMENTS,
    MESSAGE_CONDITION, MESSAGE_NAME, MESSAGE_PARTIES, MESSAGE_RESULT, MESSAGE_SCOPE,
    MESSAGE_TARGET, NativeStarted, OBJECTNAME, ObjRef, ObjectMethod, ObjectMethodWrite, Operator,
    Primitive, Raised, Resolution, SourceTaker, Started, Then, UNNAMED_METHOD, class_argument,
    compile_method_source, is_enhanced_instance, method_name_argument, pointer_address,
    request_array, required_string_argument, required_string_named_argument, scoped_method,
    unconverted_array_argument,
};

/// `RexxObject::initRexx` (`classes/ObjectClass.cpp:2546`-`:2549`): it takes
/// no arguments, does nothing, and answers `OREF_NULL`.
pub(super) fn native_no_op(
    _interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(None)
}

/// `Object~hasMethod(name)`: whether the receiver's behaviour answers `name`.
pub(super) fn native_has_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let argument = required_string_argument(interp, argument, 1)?;
    let name = String::from_utf8_lossy(&interp.to_text(argument).to_ascii_uppercase()).into_owned();
    // **The object's own dictionary answers first**, so a `setMethod` name
    // reports `1` and a hidden one reports `0` even where the class defines
    // it -- measured, oracle rc 0 for the first and rc 159 for the send
    // after the second.
    if let Some(entry) = interp.own_method_entry(receiver, name.as_bytes()) {
        return Ok(Some(interp.counted(usize::from(entry.is_some()))));
    }
    // **A receiver with no class here is loud, not `0`.** `send_message`
    // screens for it before any method runs, so this arm is unreachable
    // today; answering `0` from it anyway would make this the one place in
    // the module where a gap becomes an answer, and the answer would be one
    // the oracle contradicts.
    // **Which dictionary is asked follows the receiver**, measured on
    // `::class K` plus `::method m class`: `.K~hasMethod('M')` is `1` and
    // `.K~hasMethod('LENGTH')` is `0`, so a class object is asked about its
    // class behaviour and not about `.Class`'s instances.
    let behaviour = match interp.receiver_behaviour(receiver) {
        Ok(behaviour) => behaviour,
        Err(kind) => return Err(Loud::receiver_class(kind).into()),
    };
    let classes = &interp.object_model().classes;
    let answers = match behaviour {
        Behaviour::Instance { methods, .. } => classes.has_method_at(methods, &name),
        Behaviour::ClassSide(class) => classes.class_has_method(class, &name),
    };
    Ok(Some(interp.counted(usize::from(answers))))
}

/// `Object~defaultName`: the receiver's class id with an article in front --
/// `RexxObject::defaultNameRexx` (`classes/ObjectClass.cpp:2868`) over
/// `RexxObject::defaultName` (`:1760`).
pub(super) fn native_default_name(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(class) = native_class(interp, cleared, receiver, &[])? else {
        return Err(Loud::receiver_class("a value with no class of its own").into());
    };
    // **An enhanced object takes no article**, returning before the `a`/`an`
    // choice -- `RexxObject::defaultName` (`classes/ObjectClass.cpp:1763`).
    // Measured, oracle rc 0: `.Object~subclass('K')~enhanced(t)~defaultName`
    // is `enhanced K`.
    let enhanced = is_enhanced_instance(interp, receiver);
    let id = interp.classes().id_string(class);
    let name = if enhanced {
        format!("enhanced {id}")
    } else {
        crate::environment::default_object_name(id)
    };
    Ok(Some(interp.text_built(name.into_bytes())))
}

/// `Object~class`: the class whose behaviour answers this receiver's
/// messages -- `RexxObject::classObject` (`classes/ObjectClass.cpp:1814`),
/// whose whole body is `behaviour->getOwningClass()`.
pub(super) fn native_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let kind = match interp.receiver_kind(receiver) {
        Ok(kind) => kind,
        Err(gap) => return Err(Loud::receiver_class(gap).into()),
    };
    let model = interp.object_model();
    Ok(Some(match kind {
        Primitive::String | Primitive::SmallInt => model.string,
        Primitive::Object => model.object,
        Primitive::Array => model.array,
        Primitive::Package => model.package,
        Primitive::Method => model.method,
        Primitive::Routine => model.routine,
        Primitive::Directory => model.directory,
        Primitive::StringTable(class) => class,
        Primitive::Context => model.context,
        Primitive::StackFrame => model.stack_frame,
        Primitive::RexxInfo => model.rexx_info,
        Primitive::Message => model.message,
        Primitive::VariableReference => model.variable_reference,
        Primitive::Stem => model.stem,
        Primitive::Class(class) => model.classes.class_of(class),
        Primitive::Instance { class, .. } => class,
    }))
}

/// `Object~isA(class)`: whether the receiver's **class** is `class` or a
/// subclass of it -- `RexxObject::isInstanceOfRexx`
/// (`classes/ObjectClass.cpp:286`), which asks `classObject()` and not the
/// receiver.
pub(super) fn native_is_a(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = class_argument(interp, args)?;
    // `native_class` answers a class object for every receiver kind, so the
    // `None` a `NativeMethod` may now answer is not one of its outcomes -- and
    // a refusal rather than an `expect`, this crate's rule for an internal
    // inconsistency.
    let Some(own) = native_class(interp, cleared, receiver, &[])? else {
        return Err(Loud::receiver_class("a value with no class of its own").into());
    };
    let answer = interp.classes().is_a(own, other);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `Object~identityHash`.
pub(super) fn native_identity_hash(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let bits = receiver.bits().to_string().into_bytes();
    Ok(Some(interp.text_built(bits)))
}

/// `RexxNilObject::getHashValue` (`classes/ObjectClass.cpp:2916`), whose
/// member is stamped with this sentinel rather than derived from anything.
const NIL_HASH: u64 = 0xdead_beef;

/// The string hash `RexxString::getStringHash` computes
/// (`classes/StringClass.hpp:328`), over the bytes rather than the text.
pub(super) fn string_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for byte in bytes {
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(*byte as i8 as i64 as u64);
    }
    hash
}

/// `Object~hashCode`, `RexxObject::hashCode` (`classes/ObjectClass.cpp:398`):
/// `getHashValue()` rendered as its own eight bytes, little-endian.
pub(super) fn native_hash_code(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let value = hash_value(interp, receiver);
    Ok(Some(interp.text_built(value.to_le_bytes().to_vec())))
}

/// `RexxInternalObject::getHashValue()` as a number, which is what
/// [`native_hash_code`] renders and what the mapped collections' store hashes
/// an index by. Split out for the second caller, not changed.
pub(super) fn hash_value(interp: &mut Interp, receiver: ObjRef) -> u64 {
    if receiver == ObjRef::NIL {
        return NIL_HASH;
    }
    match interp.receiver_kind(receiver) {
        Ok(Primitive::String | Primitive::SmallInt) => string_hash(&interp.to_text(receiver)),
        Ok(Primitive::Class(class)) => string_hash(interp.classes().id_string(class).as_bytes()),
        _ => receiver.bits(),
    }
}

/// The one operand every `Object` operator method requires, or 93.903.
pub(super) fn operator_argument(args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    match args.first().copied() {
        Some(Some(argument)) => Ok(argument),
        _ => Err(Raised::missing_method_argument(1).into()),
    }
}

/// `Object~"="` and `Object~"=="`: `RexxObject::equal` and
/// `RexxObject::strictEqual` (`classes/ObjectClass.cpp:464`, `:448`), each a
/// direct identity test rather than a comparison of renderings.
pub(super) fn native_object_identical(
    _interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = operator_argument(args)?;
    Ok(Some(crate::eval::logical(receiver == other)))
}

/// `Object~"\="`, `"\=="`, `"<>"` and `"><"`: `RexxObject::notEqual` and
/// `RexxObject::strictNotEqual` (`classes/ObjectClass.cpp:492`, `:478`),
/// [`native_object_identical`]'s negation.
pub(super) fn native_object_different(
    _interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = operator_argument(args)?;
    Ok(Some(crate::eval::logical(receiver != other)))
}

/// `Object~"||"` and `Object~""`: `RexxObject::concatRexx`
/// (`classes/ObjectClass.cpp:2807`), which is `requestString()` on the
/// receiver and then that string's own concatenation.
pub(super) fn native_object_concat(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    concat_through_string_value(interp, Operator::Concatenate, receiver, args)
}

/// `Object~" "`: `RexxObject::concatBlank` (`classes/ObjectClass.cpp:2823`),
/// [`native_object_concat`] with the one separating space.
pub(super) fn native_object_concat_blank(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    concat_through_string_value(interp, Operator::Blank, receiver, args)
}

/// The shared body of the two above: the receiver's string value, then
/// [`Interp::apply_binary`] on it, which is the C++'s
/// `alias->concatRexx(otherObj)` and converts the other operand exactly as
/// every other concatenation does.
fn concat_through_string_value(
    interp: &mut Interp,
    op: Operator,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = operator_argument(args)?;
    let frame = interp.roots.activity_mut().push_frame();
    let joined = interp
        .required_string_value(receiver)
        .and_then(|alias| {
            interp.roots.activity_mut().push_temp(alias);
            interp.apply_binary(op, alias, other)
        })
        .map(Some);
    interp.roots.activity_mut().pop_frame(frame);
    joined
}

/// `Object~string`: the receiver's readable string representation --
/// `RexxObject::stringValue` (`classes/ObjectClass.cpp:1157`), which is an
/// `OBJECTNAME` send, except for a `String`, whose own override answers
/// itself.
pub(super) fn native_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // `PointerClass::stringValue` (`classes/PointerClass.cpp:152`) overrides
    // the send below: measured, oracle rc 0, `p~objectName = 'zzz'` leaves
    // `p~string` answering the address.
    if pointer_address(interp, receiver).is_some() {
        let text = interp.string_value_text(receiver);
        return Ok(Some(interp.text_built(text)));
    }
    // **Sent rather than shortcut for an instance**, whose `OBJECTNAME` a
    // program can replace: measured, oracle rc 0, `::METHOD defaultName`
    // returning `overridden` makes `o~string` answer `overridden`.
    if matches!(
        interp.receiver_kind(receiver),
        Ok(Primitive::Instance { .. })
    ) {
        let caller = interp.caller();
        let answered = interp.send_message(receiver, OBJECTNAME, None, &[], caller)?;
        if let Some(answered) = answered {
            let text = interp.string_value_text(answered);
            return Ok(Some(interp.text_built(text)));
        }
    }
    let text = interp.string_value_text(receiver);
    Ok(Some(interp.text_built(text)))
}

/// `Object~objectName`: the name something gave this object, or
/// `RexxObject::defaultName` for one nothing has named
/// (`classes/ObjectClass.cpp:1696`, `:1760`).
pub(super) fn native_object_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let kind = interp
        .receiver_kind(receiver)
        .map_err(|kind| Failure::from(Loud::receiver_class(kind)))?;
    let name = match kind {
        // `defaultName` from `.String`'s own id, which no object of this kind
        // carries a stored name for -- there is nowhere on a string to put
        // one, which is also why `native_object_name_set` refuses it.
        Primitive::String | Primitive::SmallInt => b"a String".to_vec(),
        // **A set name, or the answer to a `DEFAULTNAME` send.** The oracle
        // reads the object's own `Object`-scope variable and, for anything
        // that is not a base class, sends rather than deriving -- so a class
        // overriding `defaultName` decides what an unnamed instance answers,
        // and a name set afterwards wins over it. Both measured, oracle rc 0.
        Primitive::Instance { .. } => match interp.instance_name(receiver) {
            Some(held) => held,
            None => {
                let caller = interp.caller();
                let answered = interp.send_message(receiver, DEFAULTNAME, None, &[], caller)?;
                match answered {
                    Some(answered) => interp.string_value_text(answered),
                    None => interp.string_value_text(receiver),
                }
            }
        },
        // Derived from the class id rather than read through
        // `string_value_text`, which for this receiver answers the
        // *referenced* value: `~objectName` is the reference's own and is
        // one of the names its `UNKNOWN` never sees. Measured, oracle rc 0:
        // `vr = 5; o = >vr; say o~objectName` is `a VariableReference` where
        // `say o~string` is `5`.
        Primitive::VariableReference => {
            crate::environment::default_object_name("VariableReference").into_bytes()
        }
        // The object's own `rendered` bytes rather than `string_value_text`,
        // which for this receiver answers the **traceback line**:
        // `StackFrameClass` overrides `stringValue()` and leaves
        // `defaultName()` alone, the split `NativeObject::string_value`
        // carries. Measured, oracle rc 0, six renderings of one frame:
        // `say f`, `~string`, `~makeString` and an `Array` join are the
        // traceback line where `~objectName` and `~defaultName` are
        // `a StackFrame`. Read rather than derived, so a name
        // `~objectName=` has set still wins -- measured, `f~objectName =
        // 'tagged'` then `f~objectName` is `tagged` while `say f` is
        // unchanged.
        Primitive::StackFrame => match interp.heap.get(receiver).map(|held| &held.body) {
            Some(Body::Native(native)) => native.rendered().to_vec(),
            _ => return Err(Loud::receiver_class("a stack frame this crate did not build").into()),
        },
        // Derived for the same reason, and the two answers part here too: a
        // stem's `string_value_text` is its default. Measured, oracle rc 0:
        // `s. = 'dflt'; o = s.; say o~objectName` is `a Stem` where
        // `say o~string` is `dflt`.
        Primitive::Stem => crate::environment::default_object_name("Stem").into_bytes(),
        Primitive::Object
        | Primitive::Array
        | Primitive::Class(_)
        | Primitive::Package
        | Primitive::Method
        | Primitive::Routine
        | Primitive::Directory
        | Primitive::StringTable(_)
        | Primitive::Context
        | Primitive::RexxInfo
        | Primitive::Message => interp.string_value_text(receiver),
    };
    Ok(Some(interp.text_built(name)))
}

/// `Object~objectName=`: replaces what `~objectName` answers, and with it
/// every rendering of the object -- `RexxObject::objectNameEquals`
/// (`classes/ObjectClass.cpp:1733`), whose own return is `OREF_NULL`.
pub(super) fn native_object_name_set(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let argument = required_string_argument(interp, argument, 1)?;
    let name = interp.to_text(argument).into_owned();
    let kind = interp
        .receiver_kind(receiver)
        .map_err(|kind| Failure::from(Loud::receiver_class(kind)))?;
    match kind {
        Primitive::Class(class) => {
            let name = String::from_utf8_lossy(&name).into_owned();
            interp.classes().set_object_name(class, &name);
        }
        Primitive::Instance { .. } => {
            let Some(object) = interp.heap.get_mut(receiver) else {
                return Err(Loud::receiver_class("a value whose object is no longer live").into());
            };
            match &mut object.body {
                Body::Instance { name: held, .. } => *held = Some(name.as_slice().into()),
                other => unreachable!("an instance receiver is Body::Instance, got {other:?}"),
            }
        }
        Primitive::Package
        | Primitive::Method
        | Primitive::Routine
        | Primitive::Directory
        | Primitive::StringTable(_)
        | Primitive::Context
        | Primitive::StackFrame
        | Primitive::RexxInfo
        | Primitive::Message => {
            let Some(object) = interp.heap.get_mut(receiver) else {
                return Err(Loud::receiver_class("a value whose object is no longer live").into());
            };
            match &mut object.body {
                Body::Native(native) => native.set_rendered(&name),
                other => {
                    unreachable!("each of these receivers is Body::Native, got {other:?}")
                }
            }
        }
        Primitive::String
        | Primitive::SmallInt
        | Primitive::Object
        | Primitive::Array
        | Primitive::VariableReference
        | Primitive::Stem => {
            return Err(Loud::native_method(b"OBJECTNAME=", "Object").into());
        }
    }
    Ok(None)
}

/// `Object~setMethod(name, method, scope)`: attach one method to this object
/// alone -- `RexxObject::setMethod` (`classes/ObjectClass.cpp:1829`).
pub(super) fn native_set_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let name = method_name_argument(interp, args)?;
    let source = match args.get(1).copied().flatten() {
        None => None,
        Some(source) if interp.receiver_kind(source) == Ok(Primitive::Method) => Some(source),
        Some(source) => Some(compile_method_source(
            interp,
            &name,
            source,
            "method",
            SourceTaker::Method,
        )?),
    };
    // `newMethodObject(msgname, methobj, TheNilObject)` ahead of the checks,
    // then `defineInstanceMethod`'s own `newScope(targetScope)`
    // (`classes/ObjectClass.cpp:1847`, `:2302`): measured, oracle rc 0, the
    // method's `.context~executable` is not the object `setMethod` was given.
    let source = match source {
        None => None,
        Some(object) => {
            let (floated, _) = scoped_method(interp, object, ObjRef::NIL)?;
            interp.roots.activity_mut().push_temp(floated);
            Some(floated)
        }
    };
    let scope = set_method_scope(interp, receiver, args)?;
    check_restricted_method(interp, receiver, b"SETMETHOD")?;
    let entry = match source {
        None => None,
        Some(floated) => {
            let (executable, method) = scoped_method(interp, floated, scope)?;
            Some(ObjectMethod {
                method,
                scope,
                executable,
            })
        }
    };
    interp.write_object_method(receiver, &name, ObjectMethodWrite::Set(entry))?;
    Ok(None)
}

/// `Object~unsetMethod(name)`: take back a `setMethod` definition --
/// `RexxObject::unsetMethod` (`classes/ObjectClass.cpp:1891`).
pub(super) fn native_unset_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let name = method_name_argument(interp, args)?;
    check_restricted_method(interp, receiver, b"UNSETMETHOD")?;
    interp.write_object_method(receiver, &name, ObjectMethodWrite::Remove)?;
    Ok(None)
}

/// `setMethod`'s third argument: which variable pool the new method's
/// `EXPOSE` reaches (D67).
fn set_method_scope(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let Some(Some(option)) = args.get(2).copied() else {
        return Ok(ObjRef::NIL);
    };
    let option = required_string_named_argument(interp, option, "scope option")?;
    let text = interp.to_text(option).to_vec();
    if text.eq_ignore_ascii_case(b"OBJECT") {
        return interp
            .class_of_value(receiver)
            .ok_or_else(|| Loud::receiver_class("a value with no class of its own").into());
    }
    if text.eq_ignore_ascii_case(b"FLOAT") {
        return Ok(ObjRef::NIL);
    }
    Err(Raised::not_one_of(3, b"\"FLOAT\" or \"OBJECT\"", &text).into())
}

/// `RexxObject::checkRestrictedMethod` (`classes/ObjectClass.cpp:697`), the
/// fourth access check (D66): `run`, `setMethod` and `unsetMethod` may be
/// sent only from a method of the receiving object itself or from a class
/// method of a class it is an instance of.
fn check_restricted_method(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<(), Failure> {
    let caller = interp.caller();
    // `sender == OREF_NULL` at `:715`-`:719`, a routine or program context,
    // which the private check refuses first: `run`, `setMethod` and
    // `unsetMethod` are each private, so `check_private`'s own
    // `caller.receiver()` arm has already answered for a caller with no
    // receiver.
    let Some(sender) = caller.receiver() else {
        return Err(Raised::restricted_method(name).into());
    };
    if sender == receiver {
        return Ok(());
    }
    // `isOfClassType(Class, sender)` then `isInstanceOf((RexxClass *)sender)`
    // (`:722`-`:729`): a class method of a class this object is an instance
    // of.
    if interp.is_class_object(sender)
        && let Some(class) = interp.class_of_value(receiver)
        && interp.classes().is_a(class, sender)
    {
        return Ok(());
    }
    Err(Raised::restricted_method(name).into())
}

/// `Object~copy`: a new object with the receiver's methods and an equivalent
/// set of object variables holding the same values -- `RexxObject::copyRexx`
/// (`classes/ObjectClass.cpp:2879`).
pub(super) fn native_copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    copy_object(interp, receiver).map(Some)
}

/// [`native_copy`]'s copy of `receiver`, left on the temps.
fn copy_object(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    // **An `Array` and a `Stem` are copyable too**, and each keeps its own
    // behaviour: measured, `.Array~of('x','y')~copy~class~id` is `Array` and a
    // stem's is `Stem`. The clone below is what makes both deep in the way
    // upstream's virtual `copy()` is -- the slots and the tails travel with
    // the body, so appending to the copy leaves the receiver alone
    // (`a~items` 2 against `b~items` 3) and a tail written on the copy does
    // not appear on the receiver.
    let behaviour = match interp.receiver_kind(receiver) {
        Ok(Primitive::Instance { .. } | Primitive::Message) => rexx_core::BehaviourId::OBJECT,
        Ok(Primitive::Array) => rexx_core::BehaviourId::ARRAY,
        Ok(Primitive::Stem) => rexx_core::BehaviourId::STEM,
        // A value carried in its handle is its own copy (D15).
        Ok(Primitive::String | Primitive::SmallInt) if interp.heap.get(receiver).is_none() => {
            return Ok(receiver);
        }
        Ok(Primitive::String) => rexx_core::BehaviourId::STRING,
        Ok(_) => return Err(Loud::native_method(b"COPY", "Object").into()),
        Err(kind) => return Err(Loud::receiver_class(kind).into()),
    };
    // The allocation below collects first, and while the cloned body is a
    // local the collector does not walk, every value in it is reachable from
    // the receiver and from nowhere else.
    interp.roots.activity_mut().push_temp(receiver);
    let Some(source) = interp.heap.get(receiver) else {
        return Err(Loud::receiver_class("a value whose object is no longer live").into());
    };
    let mut body = source.body.clone();
    // `CompoundVariableTable::copyFrom` copies each tail's own value, so an
    // exposed tail comes over as one that holds none.
    if let Body::Stem { exposed, .. } = &mut body {
        *exposed = None;
    }
    let copy = interp.alloc_with(behaviour, body);
    interp.roots.activity_mut().push_temp(copy);
    duplicate_collection_stores(interp, copy);
    if interp.is_message(copy) {
        reset_copied_message(interp, copy);
    }
    if interp.answers_uninit(copy) {
        interp.heap.set_uninit(copy);
    }
    Ok(copy)
}

/// `MessageClass::copy` (`classes/MessageClass.cpp:182`): a copy keeps no
/// outcome and has its own array of the objects `~notify` named.
fn reset_copied_message(interp: &mut Interp, copy: ObjRef) {
    interp.remove_native_entry(copy, MESSAGE_RESULT);
    interp.remove_native_entry(copy, MESSAGE_CONDITION);
    if let Some(parties) = interp.native_entry(copy, MESSAGE_PARTIES) {
        let slots = interp.array_slots_of(parties).unwrap_or_default();
        let parties = interp.alloc_with(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots,
            },
        );
        interp.set_native_entry(copy, MESSAGE_PARTIES, parties);
    }
}

/// The pool entries a collection's CONTENTS live in, as (scope class, name).
pub(super) const COLLECTION_STORES: &[(&str, &[u8])] = &[
    ("Array", b"ITEMS"),
    ("List", b"ITEMS"),
    ("List", b"HANDLES"),
    ("List", b"FREE"),
    ("Supplier", b"ITEMS"),
    ("Supplier", b"INDEXES"),
    ("Table", b"HASHINDEXES"),
    ("Table", b"HASHITEMS"),
    ("Table", b"HASHNEXT"),
    // A `Directory`'s method table travels with the copy and is its own --
    // `DirectoryClass::copy` copies it rather than sharing it. Measured:
    // `g = f~copy` then `g~unsetMethod('M')` leaves `f['M']` answering.
    ("Table", b"METHODINDEXES"),
    ("Table", b"METHODITEMS"),
    ("Table", b"METHODNEXT"),
];

/// Replaces each of [`COLLECTION_STORES`] on `copy` with an array of its own.
fn duplicate_collection_stores(interp: &mut Interp, copy: ObjRef) {
    for (scope_name, entry) in COLLECTION_STORES {
        let Some(scope) = interp.classes().lookup(scope_name) else {
            continue;
        };
        let held = match interp.heap.get(copy).map(|object| &object.body) {
            Some(Body::Instance { pools, .. }) => pools.get(scope, entry),
            _ => return,
        };
        let Some(held) = held else { continue };
        let Some(slots) = interp.array_slots_of(held) else {
            continue;
        };
        let fresh = interp.alloc_with(BehaviourId::ARRAY, Body::array(slots));
        interp.roots.activity_mut().push_temp(fresh);
        interp.set_pool_variable(copy, scope, entry, fresh);
    }
}

/// `Object~run(method [, option [, argument ...]])`: run a method on this
/// object as if the object had defined it -- `RexxObject::run`
/// (`classes/ObjectClass.cpp:2185`).
pub(super) fn native_run(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(source)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method").into());
    };
    let (executable, method) = run_method(interp, source)?;
    interp.roots.activity_mut().push_temp(executable);
    let values = run_arguments(interp, args)?;
    check_restricted_method(interp, receiver, b"RUN")?;
    // Measured, oracle rc 0: `.context~executable` in a method `run` ran is
    // the object `run` was given the first time and a copy after.
    invoke_executable(
        interp,
        method,
        executable,
        receiver,
        UNNAMED_METHOD,
        &values,
    )
}

/// Runs `method` on `receiver` under the `.nil` scope as the send of `name`,
/// where no dictionary entry names the `Method` object `executable` it runs
/// for: the activation carries that object for `.context~executable`.
pub(super) fn invoke_executable(
    interp: &mut Interp,
    method: rexx_core::MethodId,
    executable: ObjRef,
    receiver: ObjRef,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let resolution = Resolution {
        scope: ObjRef::NIL,
        method,
    };
    let started = interp.begin_invoke(resolution, None, receiver, name, args)?;
    if let Started::Entered = started
        && interp.method_bodies.contains_key(&method)
    {
        interp.activation_mut().cold_mut().executable = Some(executable);
    }
    interp.complete_send(started)
}

/// The `Method` object `~run`'s first argument becomes and the identity it
/// runs under -- `MethodClass::newMethodObject(GlobalNames::RUN, methobj,
/// TheNilObject, "method")` (`classes/ObjectClass.cpp:2201`).
fn run_method(
    interp: &mut Interp,
    source: ObjRef,
) -> Result<(ObjRef, rexx_core::MethodId), Failure> {
    let object = if interp.receiver_kind(source) == Ok(Primitive::Method) {
        source
    } else {
        compile_method_source(interp, b"RUN", source, "method", SourceTaker::Method)?
    };
    scoped_method(interp, object, ObjRef::NIL)
}

/// `~run`'s `Individual`/`Array` option and the arguments behind it
/// (`classes/ObjectClass.cpp:2207`-`:2235`).
fn run_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let Some(option) = args.get(1).copied() else {
        return Ok(Vec::new());
    };
    let Some(option) = option else {
        return Err(Raised::missing_named_argument("argument style").into());
    };
    let option = required_string_named_argument(interp, option, "argument style")?;
    let text = interp.to_text(option).to_vec();
    match text.first().map(u8::to_ascii_uppercase) {
        Some(b'I') => Ok(args[2..].to_vec()),
        // `argCount < 3` and `argCount > 3` are separate raises with separate
        // numbers. Measured, oracle: `self~run('return 1', 'A')` is `88.901
        // ... argument argument array is required.` at rc 168, and a fourth
        // argument is `93.902 ... 3 expected.` at rc 163.
        Some(b'A') => {
            let Some(Some(array)) = args.get(2).copied() else {
                return Err(Raised::missing_named_argument("argument array").into());
            };
            if args.len() > 3 {
                return Err(Raised::too_many_method_arguments(3).into());
            }
            array_argument(interp, array, ArrayArgument::Named("argument array"))
        }
        _ => Err(Raised::method_option_not_recognised("AI", &text).into()),
    }
}

/// `Object~send(messagename [, argument ...])`: invoke a method on this
/// object under a name built at run time -- `RexxObject::send`
/// (`classes/ObjectClass.cpp:2008`).
pub(super) fn native_send(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let (name, scope) = decode_message_name(interp, args.first().copied().flatten())?;
    dynamic_send(interp, receiver, &name, scope, &args[1..])
}

/// `Object~sendWith(messagename, arguments)`: [`native_send`] with the
/// arguments in an array -- `RexxObject::sendWith`
/// (`classes/ObjectClass.cpp:1972`).
pub(super) fn native_send_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let (name, scope) = decode_message_name(interp, args.first().copied().flatten())?;
    let values = message_arguments(interp, args.get(1).copied().flatten())?;
    dynamic_send(interp, receiver, &name, scope, &values)
}

/// `Object~start(messagename [, argument ...])`: a `Message` object whose
/// send has been made -- `RexxObject::start`
/// (`classes/ObjectClass.cpp:2067`) through `startCommon` (`:2094`).
pub(super) fn native_start(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let Some(message) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    started_message(interp, receiver, message, &args[1..])
}

/// `Object~startWith(messagename, arguments)`: [`native_start`] with the
/// arguments in an array -- `RexxObject::startWith`
/// (`classes/ObjectClass.cpp:2046`).
pub(super) fn native_start_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let Some(message) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    let Some(arguments) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let values = array_argument(interp, arguments, ArrayArgument::Positional)?;
    started_message(interp, receiver, message, &values)
}

/// The send `~send` and `~sendWith` make, once the name has been decoded.
fn dynamic_send(
    interp: &mut Interp,
    receiver: ObjRef,
    name: &[u8],
    scope: Option<ObjRef>,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    interp.validate_scope_override(receiver, scope)?;
    let caller = interp.caller();
    Ok(
        match interp.begin_send(receiver, name, scope, args, caller)? {
            Started::Ran(value) => NativeStarted::Ran(value),
            Started::Entered => NativeStarted::Entered(Then::Pass),
        },
    )
}

/// The `Message` object `~start` and `~startWith` answer, its send made by a
/// new activity -- `RexxObject::startCommon` (`classes/ObjectClass.cpp:2094`)
/// and `MessageClass::start` (`classes/MessageClass.cpp:522`).
fn started_message(
    interp: &mut Interp,
    receiver: ObjRef,
    message: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let (name, scope) = decode_message_name(interp, Some(message))?;
    interp.validate_scope_override(receiver, scope)?;
    let class = interp.object_model().message;
    let object = interp.native_instance(class);
    interp.set_native_entry(object, MESSAGE_TARGET, receiver);
    let held_name = interp.text(&name);
    interp.set_native_entry(object, MESSAGE_NAME, held_name);
    if let Some(scope) = scope {
        interp.set_native_entry(object, MESSAGE_SCOPE, scope);
    }
    let arguments = interp.security_arguments_array(args);
    interp.set_native_entry(object, MESSAGE_ARGUMENTS, arguments);
    interp.started_messages.insert(object);
    let caller = interp.caller();
    let send = StartedSend {
        receiver,
        name,
        scope,
        args: args.to_vec(),
        caller,
    };
    spawn_send(interp, send, object)?;
    Ok(NativeStarted::Ran(Some(object)))
}

/// A started message's outcome recorded on it, and the message answered;
/// a failure that is not a condition is the start's own.
pub(super) fn record_started(
    interp: &mut Interp,
    object: ObjRef,
    sent: Result<Option<ObjRef>, Failure>,
) -> Result<Option<ObjRef>, Failure> {
    let outcome = match sent {
        Ok(None) => None,
        Ok(Some(value)) => {
            interp.set_native_entry(object, MESSAGE_RESULT, value);
            None
        }
        Err(Failure::Raised(raised)) => {
            interp.activity.failed_sends.push(object);
            Some(raised)
        }
        Err(other) => return Err(other),
    };
    let failed = outcome.is_some();
    interp.message_outcomes.insert(object, outcome);
    interp.message_completed(object);
    if !failed {
        interp.notify_parties(object)?;
    }
    Ok(Some(object))
}

/// `Message~send([receiver] [, argument ...])`: the send a `Message~new`
/// object holds, to a new receiver and with new arguments where given --
/// `MessageClass::sendRexx` (`classes/MessageClass.cpp:308`).
pub(super) fn native_message_send(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    if let Some(Some(target)) = args.first() {
        interp.set_native_entry(receiver, MESSAGE_TARGET, *target);
    }
    if args.len() > 1 {
        let arguments = interp.security_arguments_array(&args[1..]);
        interp.set_native_entry(receiver, MESSAGE_ARGUMENTS, arguments);
    }
    dispatch_held_message(interp, receiver, Then::Held(receiver))
}

/// `Message~sendWith(receiver, arguments)`: [`native_message_send`] with the
/// arguments required, in an array -- `MessageClass::sendWithRexx`
/// (`classes/MessageClass.cpp:340`).
pub(super) fn native_message_send_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    if let Some(Some(target)) = args.first() {
        interp.set_native_entry(receiver, MESSAGE_TARGET, *target);
    }
    let values = message_arguments(interp, args.get(1).copied().flatten())?;
    let arguments = interp.security_arguments_array(&values);
    interp.set_native_entry(receiver, MESSAGE_ARGUMENTS, arguments);
    dispatch_held_message(interp, receiver, Then::Held(receiver))
}

/// `MessageClass::send` and `dispatch` (`classes/MessageClass.cpp:401`,
/// `:421`): the held send made, its answer or its condition recorded for
/// `result`, `completed` and `hasError`, and the condition raised here too.
/// `then` is the work left where the send enters a Rexx activation.
fn dispatch_held_message(
    interp: &mut Interp,
    message: ObjRef,
    then: Then,
) -> Result<NativeStarted, Failure> {
    if interp.started_messages.contains(&message) {
        return Err(Raised::message_reuse().into());
    }
    let (Some(target), Some(name), Some(arguments)) = (
        interp.native_entry(message, MESSAGE_TARGET),
        interp.native_entry(message, MESSAGE_NAME),
        interp.native_entry(message, MESSAGE_ARGUMENTS),
    ) else {
        return Err(Loud::native_method(b"SEND", "Message").into());
    };
    let scope = interp.native_entry(message, MESSAGE_SCOPE);
    let name = interp.to_text(name).into_owned();
    let values = interp.array_slots_of(arguments).unwrap_or_default();
    clear_completion(interp, message);
    interp.validate_scope_override(target, scope)?;
    interp.set_notify_message(Some(message));
    let caller = interp.caller();
    let sent = match interp.begin_send(target, &name, scope, &values, caller) {
        Ok(Started::Entered) => return Ok(NativeStarted::Entered(then)),
        Ok(Started::Ran(value)) => Ok(value),
        Err(failure) => Err(failure),
    };
    record_held(interp, message, sent).map(NativeStarted::Ran)
}

/// A held send's outcome recorded on `message`, and the send's own answer.
pub(super) fn record_held(
    interp: &mut Interp,
    message: ObjRef,
    sent: Result<Option<ObjRef>, Failure>,
) -> Result<Option<ObjRef>, Failure> {
    match sent {
        Ok(answer) => {
            interp.set_notify_message(None);
            if let Some(answer) = answer {
                interp.set_native_entry(message, MESSAGE_RESULT, answer);
            }
            // An error a send of the same message made during this one stays:
            // `flagResultReturned` and `flagRaiseError` are separate flags.
            interp.message_outcomes.entry(message).or_insert(None);
            interp.message_completed(message);
            interp.notify_parties(message)?;
            Ok(answer)
        }
        // Only a `SYNTAX` condition notifies the message, and only the one in
        // the activation's single slot (`execution/RexxActivation.cpp:2470`):
        // a send made later from the same activation took it over. Any other
        // leaves the message uncompleted. The slot keeps it for the
        // activation's trap search ([`Interp::notify_slot_failure`]).
        Err(Failure::Raised(raised))
            if raised.condition == "SYNTAX" && interp.notify_slot() == Some(message) =>
        {
            interp.activity.failed_sends.push(message);
            interp
                .message_outcomes
                .insert(message, Some(raised.clone()));
            interp.message_completed(message);
            Err(Failure::Raised(raised))
        }
        Err(other) => Err(other),
    }
}

/// `MessageClass::clearCompletion` (`classes/MessageClass.cpp:382`): the
/// outcome of an earlier send forgotten.
fn clear_completion(interp: &mut Interp, message: ObjRef) {
    interp.message_outcomes.remove(&message);
    interp.unnotified_messages.remove(&message);
    interp.remove_native_entry(message, MESSAGE_RESULT);
    interp.remove_native_entry(message, MESSAGE_CONDITION);
}

/// `Message~start([receiver] [, argument ...])`: the send a `Message~new`
/// object holds, made by a new activity, to a new receiver and with new
/// arguments where given -- `MessageClass::startRexx`
/// (`classes/MessageClass.cpp:465`).
pub(super) fn native_message_start(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if let Some(Some(target)) = args.first() {
        interp.set_native_entry(receiver, MESSAGE_TARGET, *target);
    }
    if args.len() > 1 {
        let arguments = interp.security_arguments_array(&args[1..]);
        interp.set_native_entry(receiver, MESSAGE_ARGUMENTS, arguments);
    }
    start_held_message(interp, receiver)
}

/// `Message~startWith(receiver, arguments)`: [`native_message_start`] with
/// the arguments required, in an array -- `MessageClass::startWithRexx`
/// (`classes/MessageClass.cpp:497`).
pub(super) fn native_message_start_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if let Some(Some(target)) = args.first() {
        interp.set_native_entry(receiver, MESSAGE_TARGET, *target);
    }
    let values = message_arguments(interp, args.get(1).copied().flatten())?;
    let arguments = interp.security_arguments_array(&values);
    interp.set_native_entry(receiver, MESSAGE_ARGUMENTS, arguments);
    start_held_message(interp, receiver)
}

/// `MessageClass::start` (`classes/MessageClass.cpp:522`): the held send
/// handed to a new activity, which records its outcome on `message`.
fn start_held_message(interp: &mut Interp, message: ObjRef) -> Result<Option<ObjRef>, Failure> {
    if !interp.started_messages.insert(message) {
        return Err(Raised::message_reuse().into());
    }
    let (Some(target), Some(name), Some(arguments)) = (
        interp.native_entry(message, MESSAGE_TARGET),
        interp.native_entry(message, MESSAGE_NAME),
        interp.native_entry(message, MESSAGE_ARGUMENTS),
    ) else {
        return Err(Loud::native_method(b"START", "Message").into());
    };
    let scope = interp.native_entry(message, MESSAGE_SCOPE);
    let name = interp.to_text(name).into_owned();
    let args = interp.array_slots_of(arguments).unwrap_or_default();
    clear_completion(interp, message);
    interp.validate_scope_override(target, scope)?;
    let caller = interp.caller();
    let send = StartedSend {
        receiver: target,
        name,
        scope,
        args,
        caller,
    };
    spawn_send(interp, send, message)?;
    Ok(None)
}

/// Files a new activity whose first step is `send`, recording its outcome on
/// `message`.
fn spawn_send(interp: &mut Interp, send: StartedSend, message: ObjRef) -> Result<(), Failure> {
    let mut held = vec![message];
    send.object_roots(&mut held);
    for object in held {
        interp.roots.activity_mut().push_temp(object);
    }
    let spawner = interp.spawner_of_running()?;
    let mut idle = interp.new_activity();
    idle.activity.spawner = Some(spawner);
    idle.activity.first = Some(First::Send(Box::new(send)));
    idle.activity.root_then = Some(Then::Started(message));
    interp.spawn(idle);
    Ok(())
}

/// `Message~reply([receiver] [, argument ...])`: a copy of the message whose
/// send a new activity makes, to a new receiver and with new arguments where
/// given -- `MessageClass::replyRexx` (`classes/MessageClass.cpp:565`).
pub(super) fn native_message_reply(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if let Some(Some(target)) = args.first() {
        interp.set_native_entry(receiver, MESSAGE_TARGET, *target);
    }
    if args.len() > 1 {
        let arguments = interp.security_arguments_array(&args[1..]);
        interp.set_native_entry(receiver, MESSAGE_ARGUMENTS, arguments);
    }
    reply_held_message(interp, receiver)
}

/// `Message~replyWith(receiver, arguments)`: [`native_message_reply`] with
/// the arguments required, in an array -- `MessageClass::replyWithRexx`
/// (`classes/MessageClass.cpp:597`).
pub(super) fn native_message_reply_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if let Some(Some(target)) = args.first() {
        interp.set_native_entry(receiver, MESSAGE_TARGET, *target);
    }
    let values = message_arguments(interp, args.get(1).copied().flatten())?;
    let arguments = interp.security_arguments_array(&values);
    interp.set_native_entry(receiver, MESSAGE_ARGUMENTS, arguments);
    reply_held_message(interp, receiver)
}

/// `MessageClass::reply` (`classes/MessageClass.cpp:622`): the held send
/// started on a copy of `message`, which is answered; `message` itself stays
/// reusable.
fn reply_held_message(interp: &mut Interp, message: ObjRef) -> Result<Option<ObjRef>, Failure> {
    if interp.started_messages.contains(&message) {
        return Err(Raised::message_reuse().into());
    }
    clear_completion(interp, message);
    let Some(target) = interp.native_entry(message, MESSAGE_TARGET) else {
        return Err(Loud::native_method(b"REPLY", "Message").into());
    };
    let scope = interp.native_entry(message, MESSAGE_SCOPE);
    interp.validate_scope_override(target, scope)?;
    let copy = copy_object(interp, message)?;
    start_held_message(interp, copy)?;
    Ok(Some(copy))
}

/// `Message~notify(target)`: `target` is sent `messageComplete` when the send
/// completes, and at once where it already has -- `MessageClass::notify`
/// (`classes/MessageClass.cpp:209`).
pub(super) fn native_message_notify(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    const ARGUMENT: &str = "notification target";
    let Some(target) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument(ARGUMENT).into());
    };
    let Some(notification) = interp.rexx_package_class(b"MESSAGENOTIFICATION") else {
        return Err(Loud::native_method(b"NOTIFY", "Message").into());
    };
    let notifiable = interp
        .class_of_value(target)
        .is_some_and(|class| interp.classes().is_a(class, notification));
    if !notifiable {
        return Err(Raised::argument_not_an_instance(ARGUMENT, "MessageNotification").into());
    }
    let parties = match interp.native_entry(receiver, MESSAGE_PARTIES) {
        Some(parties) => parties,
        None => {
            let parties = interp.alloc_with(
                BehaviourId::ARRAY,
                Body::Array {
                    dimensions: None,
                    slots: Vec::new(),
                },
            );
            interp.set_native_entry(receiver, MESSAGE_PARTIES, parties);
            parties
        }
    };
    if let Some(Body::Array { slots, .. }) = interp.heap.get_mut(parties).map(|held| &mut held.body)
    {
        slots.push(Some(target));
    }
    let notified = interp.message_outcomes.contains_key(&receiver)
        && !interp.unnotified_messages.contains(&receiver);
    if notified {
        let caller = interp.caller();
        interp.send_message(target, MESSAGE_COMPLETE, None, &[Some(receiver)], caller)?;
    }
    Ok(None)
}

/// `Message~messageComplete(source)` and `Message~triggered(source)`: the held
/// send made, answering nothing -- `MessageClass::messageCompleted`
/// (`classes/MessageClass.cpp:693`), which ignores `source`.
pub(super) fn native_message_message_complete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    Ok(
        match dispatch_held_message(interp, receiver, Then::Triggered(receiver))? {
            NativeStarted::Ran(_) => NativeStarted::Ran(None),
            entered => entered,
        },
    )
}

/// The message each object `~notify` named is sent.
const MESSAGE_COMPLETE: &[u8] = b"MESSAGECOMPLETE";

impl Interp {
    /// Puts `message` in the running activation's notify slot and answers
    /// what it held; a native call above that activation has no slot
    /// (`ActivationBase::setObjNotify`), and neither has an activity with no
    /// activation.
    fn set_notify_message(&mut self, message: Option<ObjRef>) -> Option<ObjRef> {
        if !self.has_notify_slot() {
            return None;
        }
        let running = self.activity.running.as_deref_mut()?;
        std::mem::replace(&mut running.notify_message, message)
    }

    /// What the running activation's notify slot holds.
    fn notify_slot(&self) -> Option<ObjRef> {
        if !self.has_notify_slot() {
            return None;
        }
        self.activity.running.as_deref()?.notify_message
    }

    /// Whether the top frame is a Rexx activation, which has a notify slot.
    fn has_notify_slot(&self) -> bool {
        let Some(running) = self.activity.running.as_deref() else {
            return false;
        };
        !self
            .activity
            .native_handles
            .last()
            .is_some_and(|native| native.caller == Some(running.id))
    }

    /// `RexxActivation::trap`'s `notifyObject->error` (`execution/RexxActivation.cpp:2470`)
    /// as a `SYNTAX` failure reaches the running activation: the message in
    /// its slot, failed by this condition, is given the condition object
    /// and its notifiers run, the levels the failure left below them. A
    /// notifier's failure is answered, to be offered in place of this one.
    pub(crate) fn notify_slot_failure(&mut self, raised: &Raised) -> Option<Failure> {
        let message = self
            .activity
            .running
            .as_deref_mut()?
            .notify_message
            .take()?;
        if !self.activity.failed_sends.contains(&message)
            || self.native_entry(message, MESSAGE_PARTIES).is_none()
        {
            return None;
        }
        if let Err(failure) = self.settle_failed_sends(raised) {
            return Some(failure);
        }
        self.notify_failed_send(message, true).err()
    }

    /// Whether `object` is a `Message` this crate built.
    pub(crate) fn is_message(&mut self, object: ObjRef) -> bool {
        let message = self.object_model().message;
        matches!(
            self.heap.get(object).map(|held| &held.body),
            Some(Body::Native(native)) if native.class() == message
        )
    }

    /// `MessageClass::sendNotification` (`classes/MessageClass.cpp:645`) once
    /// the waiters are woken: `messageComplete` sent to each object `~notify`
    /// had named when it began, in order. `message` stays unnotified where
    /// one of them fails.
    pub(crate) fn notify_parties(&mut self, message: ObjRef) -> Result<(), Failure> {
        if let Some(parties) = self.native_entry(message, MESSAGE_PARTIES) {
            self.unnotified_messages.insert(message);
            let frame = self.roots.activity_mut().push_frame();
            self.roots.activity_mut().push_temp(message);
            self.roots.activity_mut().push_temp(parties);
            let count = self.array_slots(parties).map_or(0, <[_]>::len);
            let mut sent = Ok(None);
            for index in 0..count {
                let party = self
                    .array_slots(parties)
                    .and_then(|slots| slots.get(index).copied().flatten());
                let Some(party) = party else {
                    continue;
                };
                let caller = self.caller();
                sent = pinned!(
                    self,
                    crate::pinning::PinKind::Notification,
                    self.send_message(party, MESSAGE_COMPLETE, None, &[Some(message)], caller)
                );
                if sent.is_err() {
                    break;
                }
            }
            self.roots.activity_mut().pop_frame(frame);
            sent?;
            self.unnotified_messages.remove(&message);
        }
        Ok(())
    }
}

/// `Message~result`: the value the send answered, `.nil` for one that
/// answered none, and the send's own condition raised again where it failed
/// -- `MessageClass::result` (`classes/MessageClass.cpp:279`). A send not yet
/// completed parks the activity until it is.
pub(super) fn native_message_result(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    park_point!(interp, crate::pinning::ParkKind::MessageResult);
    if interp.message_outcomes.contains_key(&receiver) {
        return message_result(interp, receiver).map(NativeStarted::Ran);
    }
    check_message_deadlock(interp, receiver)?;
    let id = interp.activities.message_id(receiver);
    interp.park_native(ParkReason::MessageResult(id), message_result, receiver)
}

/// 98.905 where the activity running `message`'s send waits, through a
/// chain of waits, on the running one (`MessageClass::wait`,
/// `classes/MessageClass.cpp:250-253`).
fn check_message_deadlock(interp: &Interp, message: ObjRef) -> Result<(), Failure> {
    match interp.message_runner(message) {
        Some(runner) if interp.deadlocks(runner) => Err(crate::error::Raised::deadlock().into()),
        _ => Ok(()),
    }
}

/// [`native_message_result`] once the send has completed. A send made again
/// between the wake and this read has cleared the outcome, which answers
/// `.nil`.
fn message_result(interp: &mut Interp, message: ObjRef) -> Result<Option<ObjRef>, Failure> {
    match interp.message_outcomes.get(&message) {
        Some(Some(raised)) => {
            let raised = raised.clone();
            let Some(object) = interp.native_entry(message, MESSAGE_CONDITION) else {
                return Err(Failure::Raised(raised));
            };
            // `Activity::display` prints the object's `TRACEBACK`
            // (`concurrency/Activity.cpp:1417`).
            let lines = match interp.condition_entry(object, b"TRACEBACK") {
                Some(traceback) => super::collection::list_items(interp, traceback)?,
                None => Vec::new(),
            };
            let lines = lines
                .into_iter()
                .map(|line| interp.string_value_text(line))
                .collect();
            interp.reraise_kept_condition(*raised, object, lines)
        }
        Some(None) => {
            let held = interp.native_entry(message, MESSAGE_RESULT);
            Ok(Some(held.unwrap_or(ObjRef::NIL)))
        }
        None => Ok(Some(ObjRef::NIL)),
    }
}

/// `Message~wait`: nothing, once the send has completed --
/// `MessageClass::wait` (`classes/MessageClass.cpp:240`).
pub(super) fn native_message_wait(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    park_point!(interp, crate::pinning::ParkKind::MessageWait);
    if interp.message_outcomes.contains_key(&receiver) {
        return Ok(NativeStarted::Ran(None));
    }
    check_message_deadlock(interp, receiver)?;
    let id = interp.activities.message_id(receiver);
    interp.park_native(ParkReason::MessageWait(id), |_, _| Ok(None), receiver)
}

/// `Message~completed`: whether the send has ended, with a result or with an
/// error -- `MessageClass::completed` (`classes/MessageClass.cpp:722`), which
/// is `resultReturned() || raiseError()`.
pub(super) fn native_message_completed(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let completed = interp.message_outcomes.contains_key(&receiver);
    Ok(Some(interp.counted(usize::from(completed))))
}

/// `Message~hasError`: whether the send ended by raising --
/// `MessageClass::hasError` (`classes/MessageClass.cpp:736`).
pub(super) fn native_message_has_error(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let failed = matches!(interp.message_outcomes.get(&receiver), Some(Some(_)));
    Ok(Some(interp.counted(usize::from(failed))))
}

/// `Message~halt(description)`: asks the activity a start gave the message
/// to raise `HALT` -- `MessageClass::halt` (`classes/MessageClass.cpp:806`).
pub(super) fn native_message_halt(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let description = match args.first().copied().flatten() {
        Some(value) => {
            let text = required_string_named_argument(interp, value, "description")?;
            Some(interp.to_text(text).into_owned())
        }
        None => None,
    };
    let halted = interp.halt_message(receiver, description);
    Ok(Some(interp.counted(usize::from(halted))))
}

/// `Message~hasResult`: whether the send has answered a value --
/// `MessageClass::hasResult` (`classes/MessageClass.cpp:750`).
pub(super) fn native_message_has_result(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let answered = interp.native_entry(receiver, MESSAGE_RESULT).is_some();
    Ok(Some(interp.counted(usize::from(answered))))
}

/// `Message~errorCondition`: the condition object the send failed with, or
/// `.nil` -- `MessageClass::errorCondition` (`classes/MessageClass.cpp:764`).
pub(super) fn native_message_error_condition(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.hash_entry_read(receiver, MESSAGE_CONDITION)))
}

/// `Message~target` -- `MessageClass::messageTarget`
/// (`classes/MessageClass.cpp:777`).
pub(super) fn native_message_target(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.hash_entry_read(receiver, MESSAGE_TARGET)))
}

/// `Message~messageName` -- `MessageClass::messageName`
/// (`classes/MessageClass.cpp:789`).
pub(super) fn native_message_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.hash_entry_read(receiver, MESSAGE_NAME)))
}

/// `Message~arguments`: a copy of the argument array --
/// `MessageClass::arguments` (`classes/MessageClass.cpp:800`).
pub(super) fn native_message_arguments(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let held = interp.hash_entry_read(receiver, MESSAGE_ARGUMENTS);
    let slots = interp.array_slots_of(held).unwrap_or_default();
    Ok(Some(interp.security_arguments_array(&slots)))
}

/// `RexxObject::decodeMessageName` (`classes/ObjectClass.cpp:2125`): a
/// message name, or a two-item array of a name and the class to start the
/// method search from.
pub(super) fn decode_message_name(
    interp: &mut Interp,
    message: Option<ObjRef>,
) -> Result<(Vec<u8>, Option<ObjRef>), Failure> {
    let Some(message) = message else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    // `isString(message)` first, so a value that merely has a string value --
    // `.nil` is the reachable one -- takes the array path and its own 93.972
    // rather than being converted.
    if matches!(
        interp.receiver_kind(message),
        Ok(Primitive::String | Primitive::SmallInt)
    ) {
        return Ok((interp.to_text(message).to_ascii_uppercase(), None));
    }
    let Some(slots) = interp.array_slots_of(message) else {
        let rendered = interp.string_value_text(message);
        return Err(Raised::message_name_shape(&rendered).into());
    };
    // `messageArgCount() != 2` (`classes/ObjectClass.cpp:2143`), which is
    // `lastItem` (`classes/ArrayClass.hpp:305`) and not the slot count: a
    // trailing empty slot is not an element. Measured, oracle rc 163,
    // `o~send(('M',))` is 93.946 where the same array is `~size` 2.
    let slots = &slots[..message_argument_count(&slots)];
    if slots.len() != 2 {
        return Err(Raised::message_array_shape().into());
    }
    // `stringArgument` distinguishes an empty slot from a value with no
    // string value. Measured, oracle: `o~send((, .K))` is `88.901 ...
    // argument message name is required.` and `o~send((.nil, .K))` is
    // `88.909 Argument message name must have a string value.`, both rc 168.
    let Some(name) = slots[0] else {
        return Err(Raised::missing_named_argument("message name").into());
    };
    let name = required_string_named_argument(interp, name, "message name")?;
    let name = interp.to_text(name).to_ascii_uppercase();
    let scope = slots[1].filter(|scope| interp.is_class_object(*scope));
    if scope.is_none() {
        return Err(Raised::argument_not_a_class("SCOPE").into());
    }
    Ok((name, scope))
}

/// `ArrayClass::messageArgCount` (`classes/ArrayClass.hpp:305`), which is the
/// array's `lastItem`: the position of the last filled slot, so a trailing
/// empty slot is not an argument and an interior one is an omitted argument.
fn message_argument_count(slots: &[Option<ObjRef>]) -> usize {
    slots
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |at| at + 1)
}

/// [`message_argument_count`] applied to an argument array, which is what
/// `messageArgs()`/`messageArgCount()` hand a send together.
fn message_argument_slots(slots: Vec<Option<ObjRef>>) -> Vec<Option<ObjRef>> {
    let count = message_argument_count(&slots);
    let mut slots = slots;
    slots.truncate(count);
    slots
}

/// The argument array `~sendWith` requires -- `arrayArgument(args, "message
/// arguments")` (`classes/ObjectClass.cpp:1980`).
fn message_arguments(
    interp: &mut Interp,
    arguments: Option<ObjRef>,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let Some(arguments) = arguments else {
        return Err(Raised::missing_named_argument("message arguments").into());
    };
    array_argument(interp, arguments, ArrayArgument::Named("message arguments"))
}

/// Which `arrayArgument` overload a site calls, which is what decides the
/// refusal a multi-dimensional array takes.
pub(super) enum ArrayArgument {
    /// `arrayArgument(object, const char *name)`
    /// (`runtime/MethodArguments.hpp:703`), whose raise names the argument.
    Named(&'static str),
    /// `arrayArgument(object, size_t position)`
    /// (`runtime/MethodArguments.hpp:675`), whose raise renders the object.
    Positional,
}

/// `arrayArgument`: an argument's message-argument slots.
pub(super) fn array_argument(
    interp: &mut Interp,
    value: ObjRef,
    overload: ArrayArgument,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let value = request_array(interp, value)?;
    let Some(slots) = interp.array_slots_of(value) else {
        return Err(unconverted_array_argument(interp, value));
    };
    if interp.is_multi_dimensional_array(value) {
        return Err(match overload {
            ArrayArgument::Named(argument) => {
                Raised::argument_not_single_dimensional(argument).into()
            }
            ArrayArgument::Positional => {
                let found = interp.string_value_text(value);
                Raised::object_not_single_dimensional(&found).into()
            }
        });
    }
    Ok(message_argument_slots(slots))
}

/// `Object~request(class)`: the receiver converted to `class`, or `.nil` --
/// `RexxObject::requestRexx` (`classes/ObjectClass.cpp:1912`).
pub(super) fn native_request(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    // `stringArgument(name, ARG_ONE)->upper()`, which is `requestRexx`'s own
    // first line -- so a class name with no string value is 88.909 here and
    // not `.nil`.
    let argument = required_string_argument(interp, argument, 1)?;
    let wanted = interp.to_text(argument).to_ascii_uppercase();
    let mut make = b"MAKE".to_vec();
    make.extend_from_slice(&wanted);
    if interp.lookup(receiver, &make, None).is_some() {
        let caller = interp.caller();
        // `resultOrNil`: a `MAKE` method that returns nothing answers `.nil`
        // rather than leaving the send with no value.
        let sent = interp.send_message(receiver, &make, None, &[], caller)?;
        return Ok(Some(sent.unwrap_or(ObjRef::NIL)));
    }
    Ok(Some(match interp.receiver_class_id(receiver) {
        Some(id) if id.as_bytes().to_ascii_uppercase() == wanted => receiver,
        Some(_) | None => ObjRef::NIL,
    }))
}

/// `Object~isNil`: `1` for `.nil` and `0` for everything else.
pub(super) fn native_is_nil(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(usize::from(receiver == ObjRef::NIL))))
}
