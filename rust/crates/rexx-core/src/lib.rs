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

//! The Rexx object model: handles, the arena heap, tracing, and behaviours.

mod behaviour;
mod body;
mod handle;
mod heap;
mod roots;

pub use behaviour::{BehaviourTable, MethodId};
pub use body::{BehaviourId, Body, NotNumeric, Object};
pub use handle::{Decoded, GENERATION_MAX, ObjRef, SMALL_INT_MAX, SMALL_INT_MIN};
pub use heap::{CollectStats, Heap};
pub use roots::{FrameId, RootSet, SlotFrame};
