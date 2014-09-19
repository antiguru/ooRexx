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
/******************************************************************************/
/*                                                                            */
/* Primitive Kernel StackFrame class definitions                              */
/*                                                                            */
/******************************************************************************/
#ifndef Included_StackFrameClass
#define Included_StackFrameClass

#include "RoutineClass.hpp"
#include "MethodClass.hpp"

class PackageClass;
class ArrayClass;

class StackFrameClass : public RexxObject
{
public:
    void *operator new(size_t);

    StackFrameClass(const char *type, RexxString *name, BaseExecutable *p, RexxObject *target, ArrayClass *arguments, RexxString *t, size_t l);
    inline StackFrameClass(RESTORETYPE restoreType) { ; };

    virtual void live(size_t);
    virtual void liveGeneral(MarkReason reason);
    virtual void flatten(Envelope*);

    static void createInstance();
    static RexxClass *classInstance;

    RexxString *getType();
    RexxString *getName();
    RexxObject *getExecutable();
    RexxObject *getTarget();
    RexxObject *getLine();
    RexxString *getTraceLine();
    ArrayClass  *getArguments();
    PackageClass *getPackageObject();
    virtual     RexxString  *makeString();
    virtual     RexxString  *stringValue();

    RexxObject *newRexx(RexxObject **args, size_t argc);

    // frame identifier constants
    static const char *FRAME_COMPILE;
    static const char *FRAME_ROUTINE;
    static const char *FRAME_METHOD;
    static const char *FRAME_INTERNAL_CALL;
    static const char *FRAME_INTERPRET;
    static const char *FRAME_PROGRAM;

protected:

    const char *type;               // the type of frame
    RexxString *name;               // the name of the item at that stack frame instance
    BaseExecutable *executable;     // the executable associated with this frame instance
    RexxObject *target;             // the target object, if a message send
    ArrayClass *arguments;           // arguments to the method/routine
    size_t          line;           // the frame line position (MAX_SIZE indicates no line available)
    RexxString *traceLine;          // a tracing line
};

#endif


