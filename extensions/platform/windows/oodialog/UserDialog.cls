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
 * User Dialog Class.
 */

-- This class extends the 'PlainBaseDialog' class.  Because of backwards
-- compatibility we need to do this through the deprecated BaseDialog.
-- BaseDialog will be removed in the future.  Do NOT use BaseDialog in any code.
::class 'UserDialog' subclass BaseDialog public inherit DynamicDialog

::method init external "LIBRARY oodialog userdlg_init"

-- Called from createStaticText() in oodUser.cpp
::method setStaticTextAttribute                       -- Internal use only, do not document
  use arg id, text

  atrName = self~dataConnection[id]
  if atrName \== .nil then self~sendWith(atrName || '=', .array~of(text))

  if self~useStem then self~dlgData[id] = text

::method test external "LIBRARY oodialog userdlg_test"

-- DEPRECATED menu bar methods.  Use the Menu classes directly.
::method setMenu
    menuBar = self~menuBar

    .local~systemErrorCode = 0
    if menuBar == .nil then do
      .local~systemErrorCode = 1 -- ERROR_INVALID_FUNCTION (1) -> Incorrect function.
      return .false
    end

    if menuBar~isA(.UserMenuBar) then do
      if \ menuBar~complete then do
        self~menuBar = .nil
        return .false
      end
    end

    if \ menuBar~isA(.UserMenuBar), \ menuBar~isA(.ScriptMenuBar) then do
      .local~systemErrorCode = 1 -- ERROR_INVALID_FUNCTION (1) -> Incorrect function.
      return .false
    end

    return menuBar~attachTo(self)

::method createMenu
    use strict arg count = 100

    -- menuBar is either good, or a condition was raised.
    menuBar = .UserMenuBar~new(-1, 0, count)
    self~menuBar = menuBar
    .constDir~putAll(self~constDir)
    return .true;

::method addPopupMenu
    use strict arg name, opt = ""

    .local~systemErrorCode = 0
    if self~menuBar == .nil then do
      .local~systemErrorCode = 1 -- ERROR_INVALID_FUNCTION (1) -> Incorrect function.
      return .false
    end

    return self~menuBar~addPopup(0, name, opt, 0)

::method addMenuItem
    use strict arg name, id, opt = "", msgToRaise = .nil

    .local~systemErrorCode = 0
    if self~menuBar == .nil then do
      .local~systemErrorCode = 1 -- ERROR_INVALID_FUNCTION (1) -> Incorrect function.
      return .false
    end

    if arg(4, 'O') then return self~menuBar~addItem(id, name, opt)
    else return self~menuBar~addItem(id, name, opt, msgToRaise)

::method addMenuSeparator

    .local~systemErrorCode = 0
    if self~menuBar == .nil then do
      .local~systemErrorCode = 1 -- ERROR_INVALID_FUNCTION (1) -> Incorrect function.
      return .false
    end

    return self~menuBar~addSeparator(0)

::method loadMenu protected
  use strict arg resfile, menuid, loadopts = "", count = 50

  connect = .false
  if loadopts~caselessWordPos("CONNECTITEMS") <> 0 then connect = .true

  -- menuBar is either good, or a condition was raised.
  menuBar = .ScriptMenuBar~new(resFile, menuid, 0, count, connect)
  self~menuBar = menuBar
  .constDir~putAll(self~constDir)
  return 0
