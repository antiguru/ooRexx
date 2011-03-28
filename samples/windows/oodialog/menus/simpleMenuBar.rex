/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2011-2011 Rexx Language Association. All rights reserved.    */
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
 * A simple UserDialog with a menu, an Edit control and an UpDown control.  The
 * menu is used to change styles or behaviour of the two controls.
 *
 * About half of the menu items pop up a dialog to collect user input needed to
 * carry out the menu item action.
 *
 * This example uses a number of up-down controls in the different dialogs,
 * making it a good example of how to use up-down controls in addition, to being
 * a menu bar example.
 */

  dlg = .SimpleDialog~new
  if dlg~initCode <> 0 then do
    return 99
  end

  dlg~execute("SHOWTOP")

return 0

::requires "ooDialog.cls"

-- We need 10 digits to work with the numbers in the full range of an up-down
-- control (-2147,483,648 to 2,147,483,647)
::options digits 10

::class 'SimpleDialog' subclass UserDialog

::constant DEFAULT_TEXT   "1, 2, 3, the Edit control Menu actions work better with text in the 1st edit control."
::constant WICKED_TEXT    "The wicked flee when none pursueth ..."
::constant LOTUS_TEXT     "Lotus 123 had its ups and downs."
::constant IDES_TEXT      "Ides of March - name of March 15 in Roman calendar."
::constant TITANIC_TEXT   "472 lifeboat seats not used when 1,503 people died on the Titanic."


::method init
  expose menuBar

  forward class (super) continue

  self~addSymbolicIDs

  if \ self~createMenuBar then do
    self~initCode = 1
    return
  end

  self~makeMenuItemConnections

  self~create(30, 30, 225, 150, "Menu Bar Example", "CENTER")


::method defineDialog

  self~createStatic(IDC_ST_EDIT, 10, 32, 40, 12, "TEXT RIGHT", "Enter Text:")
  self~createEdit(IDC_EDIT, 52, 30, 160, 12, "AUTOSCROLLH KEEPSELECTION")

  self~createStatic(IDC_ST_UPD, 10, 63, 40, 12, "TEXT RIGHT", "Spin Me:")
  self~createEdit(IDC_EDIT_BUDDY, 52, 60, 65, 14, "RIGHT NUMBER")
  self~createUpDown(IDC_UPD, 257, 66, 12, 16, "WRAP ARROWKEYS AUTOBUDDY SETBUDDYINT")

  self~createPushButton(IDOK, 107, 115, 50, 14, "DEFAULT", "Ok")
  self~createPushButton(IDCANCEL, 162, 115, 50, 14, , "Cancel")

::method initDialog
  expose menuBar edit upDown

  upDown = self~newUpDown(IDC_UPD)
  upDown~setRange(1, 20000)
  upDown~setPosition(1000)

  if \ menuBar~attachTo(self) then do
    msg = "Failed to attach menu bar System Error Code:" .SystemErrorCode
    z = MessageDialog(msg, self~hwnd, "Menu Error", "OK", "WARNING")
  end

  edit = self~newEdit(IDC_EDIT)
  edit~setText(.SimpleDialog~DEFAULT_TEXT)

  self~setRadioChecks(ID_EDITCONTROL_UNRESTRICTED)


/* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/
--  The methods below, all beginning with 'on' are the implementation for each
--  of the menu item command events.
/* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/
::method onHideEdit unguarded
  expose menuBar edit

  if menuBar~isChecked(ID_FILES_HIDE_EDIT) then do
    menuBar~uncheck(ID_FILES_HIDE_EDIT)
    self~newStatic(IDC_ST_EDIT)~show
    edit~show
  end
  else do
    menuBar~check(ID_FILES_HIDE_EDIT)
    self~newStatic(IDC_ST_EDIT)~hide
    edit~hide
  end

::method onHideUpDown unguarded
  expose menuBar upDown

  if menuBar~isChecked(ID_FILES_HIDE_UPDOWN) then do
    menuBar~uncheck(ID_FILES_HIDE_UPDOWN)
    self~newStatic(IDC_ST_UPD)~show
    self~newEdit(IDC_EDIT_BUDDY)~show
    upDown~show
  end
  else do
    menuBar~check(ID_FILES_HIDE_UPDOWN)
    self~newStatic(IDC_ST_UPD)~hide
    self~newEdit(IDC_EDIT_BUDDY)~hide
    upDown~hide
  end

::method onExit unguarded
  self~cancel


::method onLower unguarded
  expose menuBar edit

  alreadyChecked = menuBar~isChecked(ID_EDITCONTROL_LOWER)

  self~setRadioChecks(ID_EDITCONTROL_LOWER)
  edit~replaceStyle("UPPER NUMBER", "LOWER")

  text = edit~getText
  edit~setText(text~lower)

  if \ alreadyChecked then do
    edit~assignFocus
    edit~select(1, 1)

    msg = "You can only enter lower case letters in" || .endOfLine || -
          "the edit control now.  Try it."
    z = MessageDialog(msg, self~hwnd, 'Edit Control Style Change', "OK", "INFORMATION")
  end


::method onNumber unguarded
  expose menuBar edit

  alreadyChecked = menuBar~isChecked(ID_EDITCONTROL_NUMBER)

  self~setRadioChecks(ID_EDITCONTROL_NUMBER)
  edit~replaceStyle("LOWER UPPER", "NUMBER")

  text = edit~getText
  edit~setText(text~translate("", xrange("00"X, "/") || xrange(":", "FF"X))~space(0))

  if \ alreadyChecked then do
    edit~assignFocus
    edit~select(1, 1)

    msg = "You can only enter numbers in the" || .endOfLine || -
          "edit control now.  Try it."
    z = MessageDialog(msg, self~hwnd, 'Edit Control Style Change', "OK", "INFORMATION")
  end


::method onUpper unguarded
  expose menuBar edit

  alreadyChecked = menuBar~isChecked(ID_EDITCONTROL_UPPER)

  self~setRadioChecks(ID_EDITCONTROL_UPPER)
  edit~replaceStyle("LOWER NUMBER", "UPPER")

  text = edit~getText
  edit~setText(text~upper)

  if \ alreadyChecked then do
    edit~assignFocus
    edit~select(1, 1)

    msg = "You can only enter upper case letters in" || .endOfLine || -
          "the edit control now.  Try it."
    z = MessageDialog(msg, self~hwnd, 'Edit Control Style Change', "OK", "INFORMATION")
  end


::method onUnRestricted unguarded
  expose menuBar edit

  alreadyChecked = menuBar~isChecked(ID_EDITCONTROL_UNRESTRICTED)

  self~setRadioChecks(ID_EDITCONTROL_UNRESTRICTED)
  edit~removeStyle("LOWER NUMBER UPPER")

  text = edit~getText
  edit~setText(.SimpleDialog~DEFAULT_TEXT)

  if \ alreadyChecked then do
    edit~assignFocus
    edit~select(1, 1)

    msg = "You can now enter unrestricted text in" || .endOfLine || -
          "the edit control.  Try it."
    z = MessageDialog(msg, self~hwnd, 'Edit Control Style Change', "OK", "INFORMATION")
  end


::method onInsert unguarded
  expose edit

  dlg = .InsertDialog~new("simpleMenuBarDialogs.rc", IDD_INSERT_DIALOG, , "simpleMenuBarDialogs.h")

  if dlg~execute("SHOWTOP", IDI_DLG_OODIALOG) == .PlainBaseDialog~IDOK then do
    edit~setText(dlg~selectedText)
  end


::method onSelect unguarded
  expose edit

  dlg = .SelectDialog~new("simpleMenuBarDialogs.rc", IDD_SELECT_DIALOG, , "simpleMenuBarDialogs.h")
  dlg~currentText = edit~getText
  edit~select(1, 1)

  if dlg~execute("SHOWTOP", IDI_DLG_APPICON) == .PlainBaseDialog~IDOK then do
    s = dlg~selection
    edit~select(s~x, s~y)
  end


::method onHexidecimal unguarded
  expose menuBar upDown

  if menuBar~isChecked(ID_UPDOWNCONTROL_HEXIDECIMAL) then do
    menuBar~uncheck(ID_UPDOWNCONTROL_HEXIDECIMAL)
    upDown~setBase(10)
  end
  else do
    menuBar~check(ID_UPDOWNCONTROL_HEXIDECIMAL)
    upDown~setBase(16)
  end


::method onSetAcceleration unguarded
  expose upDown

  dlg = .AccelDialog~new("simpleMenuBarDialogs.rc", IDD_ACCEL_DIALOG, , "simpleMenuBarDialogs.h")

  if dlg~execute("SHOWTOP", IDI_DLG_APPICON2) == .PlainBaseDialog~IDOK then do
    accel = dlg~acceleration
    upDown~setAcceleration(accel)
  end


::method onSetRange unguarded
  expose upDown

  dlg = .RangeDialog~new("simpleMenuBarDialogs.rc", IDD_RANGE_DIALOG, , "simpleMenuBarDialogs.h")

  if dlg~execute("SHOWTOP", IDI_DLG_OOREXX) == .PlainBaseDialog~IDOK then do
    r = dlg~range
    upDown~setRange(r~x, r~y)
  end


::method onPosition unguarded
  expose upDown

  dlg = .PositionDialog~new("simpleMenuBarDialogs.rc", IDD_POSITION_DIALOG, , "simpleMenuBarDialogs.h")
  dlg~upDown = upDown

  if dlg~execute("SHOWTOP", IDI_DLG_DEFAULT) == .PlainBaseDialog~IDOK then do
    p = dlg~position
    upDown~setPosition(p)
  end


::method onAbout unguarded

  dlg = .AboutDialog~new("simpleMenuBarDialogs.rc", IDD_ABOUT_DIALOG, , "simpleMenuBarDialogs.h")

  dlg~execute("SHOWTOP", IDI_DLG_DEFAULT)

/* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/
--  End of the implementation methods for each of the menu item command events.
/* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/


-- Convenience method to set the radio button menu items.  The checkRadio()
-- method takes a start resource ID and an end resource, and the resource ID for
-- a single menu item within that range of IDs.  It removes the radio button
-- check mark from all the menu items in the range and adds the radio button
-- check mark to item specified by the third argument.
::method setRadioChecks private
  expose menuBar
  use strict arg item

  menuBar~checkRadio(ID_EDITCONTROL_LOWER, ID_EDITCONTROL_UNRESTRICTED, item)


-- Creates a UserMenuBar
::method createMenuBar private
  expose menuBar

  -- Create a menu bar whose constdir attribute is copied from this dialog's
  -- constdir, and has a symbolic resource ID of IDM_MENUBAR.
  menuBar = .UserMenuBar~new(IDM_MENUBAR, self)

  -- Create the menu bar template.
  menuBar~addPopup(IDM_POP_FILES, "Files")
    menuBar~addItem(ID_FILES_HIDE_EDIT, "Hide Edit Control", "DEFAULT CHECK")
    menuBar~addItem(ID_FILES_HIDE_UPDOWN, "Hide UpDown Control", " CHECK")
    menuBar~addSeparator(IDM_SEP_FILES)
    menuBar~addItem(ID_FILES_EXIT, "Exit", "END")

  menuBar~addPopup(IDM_POP_EDITCONTROL, "Edit Control")
    menuBar~addItem(ID_EDITCONTROL_LOWER, "Lower Case Only", "CHECK RADIO")
    menuBar~addItem(ID_EDITCONTROL_NUMBER, "Numbers Only", "CHECK RADIO")
    menuBar~addItem(ID_EDITCONTROL_UPPER, "Upper Case Only", "CHECK RADIO")
    menuBar~addItem(ID_EDITCONTROL_UNRESTRICTED, "No Restriction", "CHECK RADIO DEFAULT")
    menuBar~addSeparator(IDM_SEP_EDITCONTROL)
    menuBar~addItem(ID_EDITCONTROL_INSERT, "Insert Text ...")
    menuBar~addItem(ID_EDITCONTROL_SELECT, "Select Text ...", "END")

  menuBar~addPopup(IDM_POP_UPDOWNCONTROL, "UpDown Control")
    menuBar~addItem(ID_UPDOWNCONTROL_HEXIDECIMAL, "Hexidecimal", "CHECK")
    menuBar~addSeparator(IDM_SEP_UPDOWNCONTROL)
    menuBar~addItem(ID_UPDOWNCONTROL_SET_ACCELERATION, "Set Acceleration ...")
    menuBar~addItem(ID_UPDOWNCONTROL_SET_RANGE, "Set Range ...")
    menuBar~addItem(ID_UPDOWNCONTROL_SET_POSITION, "Set Position ...", "END")

  menuBar~addPopup( IDM_POP_HELP, "Help", "END")
    menuBar~addItem(ID_HELP_ABOUT, "About Simple Menu", "END")

  if \ menuBar~complete then do
    say 'User menu bar completion error:' .SystemErrorCode SysGetErrortext(.SystemErrorCode)
    return .false
  end

  return .true

-- Connect each of the command menu items with a method.
::method makeMenuItemConnections private
  expose menuBar

  menuBar~connectCommandEvent(ID_FILES_HIDE_EDIT, onHideEdit, self)
  menuBar~connectCommandEvent(ID_FILES_HIDE_UPDOWN, onHideUpDown, self)
  menuBar~connectCommandEvent(ID_FILES_EXIT, onExit, self)

  menuBar~connectCommandEvent(ID_EDITCONTROL_LOWER, onLower, self)
  menuBar~connectCommandEvent(ID_EDITCONTROL_NUMBER, onNumber, self)
  menuBar~connectCommandEvent(ID_EDITCONTROL_UPPER, onUpper, self)
  menuBar~connectCommandEvent(ID_EDITCONTROL_UNRESTRICTED, onUnRestricted, self)
  menuBar~connectCommandEvent(ID_EDITCONTROL_INSERT, onInsert, self)
  menuBar~connectCommandEvent(ID_EDITCONTROL_SELECT, onSelect, self)

  menuBar~connectCommandEvent(ID_UPDOWNCONTROL_HEXIDECIMAL, onHexiDecimal, self)
  menuBar~connectCommandEvent(ID_UPDOWNCONTROL_SET_ACCELERATION, onSetAcceleration, self)
  menuBar~connectCommandEvent(ID_UPDOWNCONTROL_SET_RANGE, onSetRange, self)
  menuBar~connectCommandEvent(ID_UPDOWNCONTROL_SET_POSITION, onPosition, self)

  menuBar~connectCommandEvent(ID_HELP_ABOUT, onAbout, self)

-- Populate the constdir with symbolic resource IDs.
--
-- This is done just to demonstrate that there is no reason why the constdir has
-- to be populated automatically through the use of a header file or #define
-- lines in a resource script.  Normally these symbolic IDs whould be in the
-- simpleMenuBarDialogs.h file.
::method addSymbolicIDs private

  self~constDir[IDC_EDIT]       = 900
  self~constDir[IDC_ST_EDIT]    = 901
  self~constDir[IDC_UPD]        = 902
  self~constDir[IDC_ST_UPD]     = 903
  self~constDir[IDC_EDIT_BUDDY] = 904

  self~constDir[IDM_MENUBAR]                       = 800
  self~constDir[IDM_POP_FILES]                     = 810
  self~constDir[ID_FILES_HIDE_EDIT]                = 811
  self~constDir[ID_FILES_HIDE_UPDOWN]              = 812
  self~constDir[IDM_SEP_FILES]                     = 813
  self~constDir[ID_FILES_EXIT]                     = 814
  self~constDir[IDM_POP_EDITCONTROL]               = 820
  self~constDir[ID_EDITCONTROL_LOWER]              = 821
  self~constDir[ID_EDITCONTROL_NUMBER]             = 822
  self~constDir[ID_EDITCONTROL_UPPER]              = 823
  self~constDir[ID_EDITCONTROL_UNRESTRICTED]       = 824
  self~constDir[IDM_SEP_EDITCONTROL]               = 825
  self~constDir[ID_EDITCONTROL_INSERT]             = 826
  self~constDir[ID_EDITCONTROL_SELECT]             = 827
  self~constDir[IDM_POP_UPDOWNCONTROL]             = 830
  self~constDir[ID_UPDOWNCONTROL_HEXIDECIMAL]      = 831
  self~constDir[IDM_SEP_UPDOWNCONTROL]             = 832
  self~constDir[ID_UPDOWNCONTROL_SET_ACCELERATION] = 833
  self~constDir[ID_UPDOWNCONTROL_SET_RANGE]        = 834
  self~constDir[ID_UPDOWNCONTROL_SET_POSITION]     = 835
  self~constDir[IDM_POP_HELP]                      = 840
  self~constDir[ID_HELP_ABOUT]                     = 841

::method initAutoDetection
  self~noAutoDetection


/* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/
--  The following classes all implement a single dialog that is used to collect
--  information for the user needed to carry out one of the menu item commands.
/* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/

::class 'InsertDialog' subclass RcDialog

::attribute selectedText

::method initDialog
  self~newRadioButton(IDC_RB_WICKED)~check

::method ok unguarded

  select
    when self~newRadioButton(IDC_RB_WICKED)~checked then self~selectedText = .SimpleDialog~WICKED_TEXT
    when self~newRadioButton(IDC_RB_LOTUS)~checked then self~selectedText = .SimpleDialog~LOTUS_TEXT
    when self~newRadioButton(IDC_RB_IDES)~checked then self~selectedText = .SimpleDialog~IDES_TEXT
    when self~newRadioButton(IDC_RB_TITANIC)~checked then self~selectedText = .SimpleDialog~TITANIC_TEXT
    otherwise self~selectedText = ""
  end
  -- End select

  self~ok:super

::method initAutoDetection
  self~noAutoDetection


::class 'SelectDialog' subclass RcDialog

::attribute selection
::attribute currentText

::method initDialog
  expose updStart updEnd currentText

  self~newStatic(IDC_ST_CURRENT_TEXT)~setText(currentText)

  updStart = self~newUpDown(IDC_UPD_START)
  updStart~setRange(0, currentText~length)
  updStart~setPosition(1)

  updEnd = self~newUpDown(IDC_UPD_END)
  updEnd~setRange(0, currentText~length)
  updEnd~setPosition(1)

::method ok unguarded
  expose updStart updEnd

  self~selection = .Point~new(updStart~getPosition, updEnd~getPosition)
  self~ok:super

::method initAutoDetection
  self~noAutoDetection


::class 'AccelDialog' subclass RcDialog

::attribute acceleration

::method initDialog

  upd = self~newUpDown(IDC_UPD_ACCEL_SECONDS0)
  upd~setPosition(0)
  upd~disable
  self~newEdit(IDC_EDIT_ACCEL_SECONDS0)~disable

  upd = self~newUpDown(IDC_UPD_ACCEL0)
  upd~setPosition(1)
  upd~disable
  self~newEdit(IDC_EDIT_ACCEL0)~disable

  do i = 1 to 3
    upd = self~newUpDown(IDC_UPD_ACCEL_SECONDS || i)
    upd~setRange(i, 32)
    upd~setPosition(i)

    upd = self~newUpDown(IDC_UPD_ACCEL || i)
    upd~setRange(2 ** i, 256)
    upd~setPosition(2 ** i)
  end

  self~newUpDown(IDC_UPD_ACCEL_SECONDS1)~assignFocus


::method ok unguarded

  a = .array~new(3)

  do i = 1 to 3
    d = .directory~new

    updS = self~newUpDown(IDC_UPD_ACCEL_SECONDS || i)
    updA = self~newUpDown(IDC_UPD_ACCEL || i)

    d~seconds = updS~getPosition
    d~increment = updA~getPosition
    a[i] = d
  end

  -- Check that the user has an unique value for each acceleration entry.
  check = .set~of(a[1]~seconds, a[2]~seconds, a[3]~seconds)
  if check~items <> 3 then do
    msg = "For each of the 3 acceleration input" || .endOfLine || -
          "lines, you must use a unique value for" || .endOfLine || -
          "seconds.  Found:" a[1]~seconds',' a[2]~seconds', and' a[3]~seconds'.'

    z = MessageDialog(msg, self~hwnd, "Acceleration Input Error", "OK", "STOP")
    return .false
  end

  -- Sort the entries by seconds, ascending, using brute force.
  max = 0; min = 4
  do i = 1 to 3
    if a[i]~seconds > max then max = i
    if a[i]~seconds < min then min = i
  end

  -- The user only has 3 up-down pairs she can set.  But, we also have the
  -- first disabled up-down pair for seconds == 0, increment == 1
  aa = .array~new(4)
  aa[1] = .directory~new~~setEntry("SECONDS", 0)~~setEntry("INCREMENT", 1)

  aa[2] = a[min]
  aa[4] = a[max]

  do i = 1 to 3
    if i \== min, i \== max then do
      aa[3] = a[i]
      leave
    end
  end

  self~acceleration = aa
  self~ok:super

::method initAutoDetection
  self~noAutoDetection


::class 'RangeDialog' subclass RcDialog

::attribute range

::method initDialog
  expose updLow updHigh

  accel = .array~new(4)
  accel[1] = .directory~new~~setEntry("SECONDS", 0)~~setEntry("INCREMENT", 1)
  accel[2] = .directory~new~~setEntry("SECONDS", 1)~~setEntry("INCREMENT", 32)
  accel[3] = .directory~new~~setEntry("SECONDS", 2)~~setEntry("INCREMENT", 64)
  accel[4] = .directory~new~~setEntry("SECONDS", 3)~~setEntry("INCREMENT", 256)

  updLow = self~newUpDown(IDC_UPD_LOW)
  updLow~setRange(-2147483648, 2147483647)
  updLow~setPosition(0)
  updLow~setAcceleration(accel)

  updHigh = self~newUpDown(IDC_UPD_HIGH)
  updHigh~setRange(-2147483648, 2147483647)
  updHigh~setPosition(0)
  updHigh~setAcceleration(accel)

::method ok unguarded
  expose updLow updHigh

  self~range = .Point~new(updLow~getPosition, updHigh~getPosition)
  self~ok:super

::method initAutoDetection
  self~noAutoDetection


::class 'PositionDialog' subclass RcDialog

::attribute position
::attribute upDown

::method initDialog
  expose upDown upd

  upd = self~newUpDown(IDC_UPD_POSITION)

  upd~setRange(upDown~getRange)
  upd~setPosition(upDown~getPosition)

::method ok unguarded
  expose upd

  self~position = upd~getPosition
  self~ok:super

::method initAutoDetection
  self~noAutoDetection


::class 'AboutDialog' subclass RcDialog

::method initDialog
   expose font

   bitmap = .Image~getImage("simpleMenuBarOORexx.bmp")
   self~newStatic(IDC_ST_BITMAP)~setImage(bitmap)

   font = self~createFontEx("Ariel", 14)
   self~newStatic(IDC_ST_ABOUT)~setFont(font)

::method leafing
   expose font
   self~deleteFont(font)
