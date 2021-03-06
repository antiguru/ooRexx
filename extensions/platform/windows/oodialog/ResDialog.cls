/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 1995, 2004 IBM Corporation. All rights reserved.             */
/* Copyright (c) 2005-2014 Rexx Language Association. All rights reserved.    */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* http://www.oorexx.org/license.html                                         */
/*                                                                            */
/* Redistribution and use in source and binary forms, with or                 */
/* without modification, are permitted provided that the following            */
/* conditions are met:                                                        */
/*                                                                            */
/* Redistributions of source code must retain the above copyright             */
/* notice, this list of conditions and the following disclaimer.              */
/* Redistributions in binary form must reproduce the above copyright          */
/* notice, this list of conditions and the following disclaimer in            */
/* the documentation and/or other materials provided with the distribution.   */
/*                                                                            */
/* Neither the name of Rexx Language Association nor the names                */
/* of its contributors may be used to endorse or promote products             */
/* derived from this software without specific prior written permission.      */
/*                                                                            */
/* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS        */
/* "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT          */
/* LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS          */
/* FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT   */
/* OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,      */
/* SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED   */
/* TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA,        */
/* OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY     */
/* OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING    */
/* NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS         */
/* SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.               */
/*                                                                            */
/*----------------------------------------------------------------------------*/

/**
 * Windows Dialog Interface for Open Object Rexx (ooRexx.)
 *
 * Resource Dialog Class.
 */

-- This class extends the 'PlainBaseDialog' class.  Because of backwards
-- compatibility we need to do this through the deprecated BaseDialog.
-- BaseDialog will be removed in the future.  Do NOT use BaseDialog in any code.
::class 'ResDialog' subclass BaseDialog public

::method init external "LIBRARY oodialog resdlg_init"

-- Documented "internal use only" in 3.2.0 - Do not document.
::method startIt
   use strict arg icon = 0, modeless = .false

   icon = self~resolveIconID(icon)
   if \ modeless~datatype('O') then modeless = .false

   if self~startDialog(self~library, self~resourceID, icon, modeless) then do
      -- if autodetection is on, create the data attributes
      if self~autoDetect then do
         ids = self~getDataTableIDs
         do id over ids
            text = self~getControlText(id)
            self~addAttribute(id, text)
         end
      end
      self~initDialog
      return .true
   end

   return .false

::method startDialog private external "LIBRARY oodialog resdlg_startDialog_pvt"
::method getDataTableIDs private external "LIBRARY oodialog resdlg_getDataTableIDs_pvt"

-- DEPRECATED
::method setMenu
  use strict arg id
  menuBar = .BinaryMenuBar~new(self, id, 0, self)
  if menuBar == 0 then return .false
  self~menuBar = menuBar
  .constDir~putAll(self~constDir)
  return .true

