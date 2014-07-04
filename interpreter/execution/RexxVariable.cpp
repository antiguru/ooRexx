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
/****************************************************************************/
/* REXX Kernel                                           RexxVariable.c     */
/*                                                                          */
/* Primitive Variable Class                                                 */
/*                                                                          */
/****************************************************************************/
#include "RexxCore.h"
#include "RexxVariable.hpp"
#include "RexxActivity.hpp"
#include "ActivityManager.hpp"

/**
 * Allocate a new variable object.
 *
 * @param size   The size of the object.
 *
 * @return Storage for creating the object.
 */
void *RexxVariable::operator new(size_t size)
{
    return new_object(size, T_Variable);
}


/**
 * Perform garbage collection on a live object.
 *
 * @param liveMark The current live mark.
 */
void RexxVariable::live(size_t liveMark)
{
    memory_mark(variableValue);
    memory_mark(variable_name);
    memory_mark(dependents);
}


/**
 * Perform generalized live marking on an object.  This is
 * used when mark-and-sweep processing is needed for purposes
 * other than garbage collection.
 *
 * @param reason The reason for the marking call.
 */
void RexxVariable::liveGeneral(int reason)
{
    memory_mark_general(variableValue);
    memory_mark_general(variable_name);
    memory_mark_general(dependents);
}


/**
 * Flatten a source object.
 *
 * @param envelope The envelope that will hold the flattened object.
 */
void RexxVariable::flatten(RexxEnvelope *envelope)
{
    setUpFlatten(RexxVariable)

     flattenRef(variableValue);
     flattenRef(variable_name);
     flattenRef(dependents);

    cleanUpFlatten
}


/**
 * Request that an activity be informed of any variable
 * modifications.
 *
 * @param informee The requesting activity.
 */
void RexxVariable::inform(RexxActivity *informee)
{
    // we don't typically have a dependents list until the
    // first time this is needed
    if (dependents == OREF_NULL)
    {
        // use an object table for this
        OrefSet(this, dependents, new_identity_table());
    }
    // add this to the table as the index
    dependents->put(TheNilObject, (RexxObject *)informee);
}



/**
 * Remove a notification watch from a variable.
 *
 * @param informee The activity requesting removal.
 */
void RexxVariable::uninform(RexxActivity *informee)
{
    // remove the entry
    dependents->remove((RexxObject *)informee);
    // It's probably a coin flip on whether this should
    // be removed when this becomes empty.  This happens
    // because a method has used GUARD WHEN to wait on a variable.
    // On the assumption that this object was written with this
    // sort of multitasking in mind, it is likely that  this
    // variable instance might be waited on again.  Therefore,
    // it is probably prudent to keep the table around to
    // prevent thrashing.  For example, something like a work
    // queue where one thread is waiting for items to be added
    // would be an example where you would be constantly adding and
    // deleting this table.
}

/**
 * Drop a variable.
 */
void RexxVariable::drop()
{
    // clear out the value
    OrefSet(this, variableValue, OREF_NULL);
    // if we have watchers, notify them
    if (dependents != OREF_NULL && !dependents->isEmpty())
    {
        notify();
    }
}

/**
 * notify all waiting activities that a variable has been updated.
 */
void RexxVariable::notify()
{
    // if we have a dependents table, iterate through the table tapping
    // the waiting activities
    if (dependents != OREF_NULL)
    {
        for (HashLink i = dependents->first(); dependents->available(i); i = dependents->next(i))
        {
            // post the event to the waiting thread
            ((RexxActivity *)dependents->index(i))->guardPost();
        }

        // yield control and allow the waiting guard(s) to run too
        RexxActivity *activity = ActivityManager::currentActivity;
        activity->releaseAccess();         // release the lock
        activity->requestAccess();         // get it back again
    }
}


/**
 * Set a variable to a stem value.  This handles all of the
 * details of stem-to-stem assignment and stem variable
 * re-initialization.
 *
 * @param value  The value to set.
 */
void RexxVariable::setStem(RexxObject *value)
{
    // if this is a stem-to-stem assignment, we replace the current variable's
    // stem object.
    if (isOfClass(Stem, value))
    {
        set(value);
    }
    // assigning a new value to the stem.  This creates a new stem object.
    else
    {
        // create a new stem object as value
        RexxStem *stem_table = new RexxStem (variable_name);
        set(stem_table);                   // overlay the reference stem object
        stem_table->setValue(value);       // set the default value
    }
}
