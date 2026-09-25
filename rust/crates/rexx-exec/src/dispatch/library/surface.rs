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

//! The [`Surface`] an extension's callbacks reach this interpreter through.

use rexx_api::callbacks::Surface;
use rexx_core::{BehaviourId, Body, ObjRef};

use crate::Interp;

impl Surface for Interp {
    fn clear_condition(&mut self) {
        self.native_frame_mut().raised = None;
    }

    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef {
        self.alloc_with(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots: items.to_vec(),
            },
        )
    }
}
