//! Measurements of the failed-send settlement interval. These handles are not GC roots.

use crate::dispatch::hash::{DirectoryEntry, Key, directory_get};
use crate::{Interp, ObjRef};
use rexx_core::Decoded;
use std::cell::RefCell;

#[derive(Debug)]
pub(crate) struct Observation {
    pub(crate) message: (u32, u32),
    pub(crate) collections: u64,
    pub(crate) alive: bool,
    pub(crate) attached: bool,
    pub(crate) code: Vec<u8>,
    pub(crate) replacement: Option<(u32, u32)>,
    pub(crate) replacement_unchanged: bool,
}

thread_local! {
    static OBSERVATIONS: RefCell<Vec<Observation>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn take() -> Vec<Observation> {
    OBSERVATIONS.with(|observations| std::mem::take(&mut *observations.borrow_mut()))
}

pub(super) struct Settlement {
    messages: Vec<ObjRef>,
    collections: u64,
}

impl Settlement {
    pub(super) fn start(interp: &Interp) -> Self {
        Self {
            messages: interp.activity.failed_sends.clone(),
            collections: interp.heap.collections_performed(),
        }
    }

    pub(super) fn before_attach(&self, interp: &Interp) -> Vec<Option<(ObjRef, String)>> {
        let collections = interp.heap.collections_performed() - self.collections;
        self.messages
            .iter()
            .map(|message| {
                if interp.heap.peek(*message).is_some() {
                    return None;
                }
                let Decoded::Heap { slot, generation } = message.decode() else {
                    return None;
                };
                // A slot's generation advances only when it is swept. The interval's
                // collection count bounds every possible reuse of this slot.
                (1..=collections).find_map(|offset| {
                    let offset = u32::try_from(offset).expect("bounded settlement collections");
                    let replacement = ObjRef::heap(slot, generation.checked_add(offset)?);
                    interp
                        .heap
                        .peek(replacement)
                        .map(|held| (replacement, format!("{held:?}")))
                })
            })
            .collect()
    }

    pub(super) fn finish(
        self,
        interp: &mut Interp,
        object: ObjRef,
        before: Vec<Option<(ObjRef, String)>>,
    ) {
        let collections = interp.heap.collections_performed() - self.collections;
        for (message, replacement) in self.messages.into_iter().zip(before) {
            let alive = interp.heap.peek(message).is_some();
            let attached = interp.native_entry(message, crate::dispatch::MESSAGE_CONDITION);
            let code = attached
                .and_then(|condition| {
                    match directory_get(interp, condition, Key::new(b"CODE"), None)
                        .ok()?
                        .0
                    {
                        DirectoryEntry::Found(value) => Some(interp.to_text(value).into_owned()),
                        _ => None,
                    }
                })
                .unwrap_or_default();
            let replacement_unchanged = replacement.as_ref().is_none_or(|(handle, before)| {
                interp
                    .heap
                    .peek(*handle)
                    .is_some_and(|held| format!("{held:?}") == *before)
            });
            let observation = Observation {
                message: identity(message),
                collections,
                alive,
                attached: attached == Some(object),
                code,
                replacement: replacement.map(|(handle, _)| identity(handle)),
                replacement_unchanged,
            };
            OBSERVATIONS.with(|observations| observations.borrow_mut().push(observation));
        }
    }
}

fn identity(handle: ObjRef) -> (u32, u32) {
    let Decoded::Heap { slot, generation } = handle.decode() else {
        panic!("a settled Message has a heap identity");
    };
    (slot, generation)
}
