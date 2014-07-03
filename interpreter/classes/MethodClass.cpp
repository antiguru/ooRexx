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
/* REXX Kernel                                                                */
/*                                                                            */
/* Primitive Method Class                                                     */
/*                                                                            */
/******************************************************************************/
#include "RexxCore.h"
#include "StringClass.hpp"
#include "ArrayClass.hpp"
#include "RexxCode.hpp"
#include "RexxNativeCode.hpp"
#include "RexxActivity.hpp"
#include "RexxActivation.hpp"
#include "RexxNativeActivation.hpp"
#include "MethodClass.hpp"
#include "SourceFile.hpp"
#include "DirectoryClass.hpp"
#include "ProtectedObject.hpp"
#include "BufferClass.hpp"
#include "RexxInternalApis.h"
#include "RoutineClass.hpp"
#include "PackageClass.hpp"
#include "Interpreter.hpp"
#include "RexxCode.hpp"
#include "PackageManager.hpp"
#include "LanguageParser.hpp"

// singleton class instance
RexxClass *MethodClass::classInstance = OREF_NULL;


/**
 * Create initial class object at bootstrap time.
 */
void MethodClass::createInstance()
{
    CLASS_CREATE(Method, "Method", RexxClass);
}


/**
 * Allocate a new Method object.
 *
 * @param size   The base size of the method object.
 *
 * @return The storage for a method object.
 */
void *MethodClass::operator new (size_t size)
{
    return new_object(size, T_Method);
}


/**
 * Initialize a Routine object from a generated code object. Generally
 * used for routines generated from ::METHOD directives.
 *
 * @param name    The routine name.
 * @param codeObj The associated code object.
 */
MethodClass::MethodClass(RexxString *name, BaseCode *codeObj)
{
    executableName = name;
    code = codeObj;
}


/**
 * Perform garbage collection on a live object.
 *
 * @param liveMark The current live mark.
 */
void MethodClass::live(size_t liveMark)
{
    memory_mark(scope);
    memory_mark(code);
    memory_mark(executableName);
    memory_mark(objectVariables);
}


/**
 * Perform generalized live marking on an object.  This is
 * used when mark-and-sweep processing is needed for purposes
 * other than garbage collection.
 *
 * @param reason The reason for the marking call.
 */
void MethodClass::liveGeneral(int reason)
{
    memory_mark_general(scope);
    memory_mark_general(code);
    memory_mark_general(executableName);
    memory_mark_general(objectVariables);
}


/**
 * Flatten a source object.
 *
 * @param envelope The envelope that will hold the flattened object.
 */
void MethodClass::flatten(RexxEnvelope *envelope)
{
    setUpFlatten(MethodClass)

     flattenRef(scope);
     flattenRef(code);
     flattenRef(executableName);
     flattenRef(objectVariables);

    cleanUpFlatten
}


/**
 * Invoke this method on an object.
 *
 * @param activity The current activity.
 * @param receiver The receiver object.
 * @param msgname  The message name.
 * @param argPtr   The argument pointer.
 * @param count    The argument count.
 * @param result   The result object used to pass a result back.
 */
void MethodClass::run(RexxActivity *activity, RexxObject *receiver, RexxString *msgname,
    RexxObject **argPtr, size_t count, ProtectedObject &result)
{
    // just forward this to the code object
    code->run(activity, this, receiver, msgname, argPtr, count, result);
}


/**
 * Change the scope of a method object.  This also returns
 * a new method object.
 *
 * @param _scope The new scope.
 *
 * @return The new method object with the new scope.
 */
MethodClass *MethodClass::newScope(RexxClass  *_scope)
{
    // if this doesn't have a scope yet, we can just override what's here
    // and return the same method instance.
    if (scope == OREF_NULL)
    {
        setField(scope, _scope);
        return this;
    }
    else
    {
        // we need to return a copy of the method with the scope set
        MethodClass *newMethod= (MethodClass *)this->copy();
        OrefSet(newMethod, newMethod->scope, _scope);
        return newMethod;
    }
}


/**
 * Associate a security manager with a method's source.
 *
 * @param manager The security manager instance.
 *
 * @return The return code from setting this on the code object.
 */
RexxObject *MethodClass::setSecurityManager(RexxObject *manager)
{
    return code->setSecurityManager(manager);
}

/**
 * Internal method to just set the scope for the method.
 *
 * @param _scope The new scope.
 */
void MethodClass::setScope(RexxClass  *_scope)
{
    setField(scope, _scope);
}


/**
 * Change the guarded state of the method from Rexx code.
 *
 * @return No return value.
 */
RexxObject *MethodClass::setUnguardedRexx()
{
    setUnguarded();
    return OREF_NULL;
}


/**
 * Change the guarded state of the method from Rexx code.
 *
 * @return No return value.
 */
RexxObject *MethodClass::setGuardedRexx( )
{
    setGuarded();
    return OREF_NULL;
}


/**
 * Flag the method as being private from Rexx code.
 *
 * @return No return value.
 */
RexxObject *MethodClass::setPrivateRexx()
{
    setPrivate();
    return OREF_NULL;
}


/**
 * Turn on a method's protected state from Rexx code.
 *
 * @return No return value.
 */
RexxObject *MethodClass::setProtectedRexx()
{
    setProtected();
    return OREF_NULL;
}


/**
 * Return the Guarded setting for a method object.
 *
 * @return .true if the method is guarded.  .false otherwise.
 */
RexxObject *MethodClass::isGuardedRexx( )
{
    return isGuarded() ? TheTrueObject : TheFalseObject;
}


/**
 * Return the Private setting for a method object.
 *
 * @return .true if the method is private.  .false otherwise.
 */
RexxObject *MethodClass::isPrivateRexx( )
{
    return isPrivate() ? TheTrueObject : TheFalseObject;
}


/**
 * Return the Protected setting for a method object.
 *
 * @return .true if the method is protected.  .false otherwise.
 */
RexxObject *MethodClass::isProtectedRexx( )
{
    return isProtected() ? TheTrueObject : TheFalseObject;
}


/**
 * Set the entire set of method attributes with one call.  Used
 * during source compilation.
 *
 * @param _private   The private setting.
 * @param _protected The protected setting.
 * @param _guarded   The guarded setting.
 */
void MethodClass::setAttributes(bool _private, bool _protected, bool _guarded)
{
    if (_private)
    {
        setPrivate();
    }
    if (_protected)
    {
        setProtected();
    }
    // guarded is the default, so we need to reverse this
    if (!_guarded)
    {
        setUnguarded();
    }
}


/**
 * Flatten a method into a buffer for saving to another
 * source (a file perhaps, or stored in the macrospace.)
 *
 * @return The smart buffer containing the flattened code.
 */
RexxSmartBuffer *MethodClass::saveMethod()
{
    RexxEnvelope *envelope = new RexxEnvelope;
    ProtectedObject p(envelope);

    // pack the method object (and source object) into the buffer.
    envelope->pack(this);
    return envelope->getBuffer();
}


/**
 * Restore a method object from the saved state.
 *
 * @param buffer The buffer object containing the flattened object.
 * @param startPointer
 *               The starting pointer within the buffer.
 * @param length The length of the flattened object.
 *
 * @return An inflated Method object.
 */
MethodClass *MethodClass::restore(RexxBuffer *buffer, char *startPointer, size_t length)
{
    // get an envelop and puff the object back to usability.
    RexxEnvelope *envelope  = new RexxEnvelope;
    ProtectedObject p(envelope);
    envelope->puff(buffer, startPointer, length);

    // The method object is the root of the object tree.  Get that from the
    // envelop and return it.
    return (MethodClass *)envelope->getReceiver();
}


/**
 * Static method used for constructing new method objects in
 * various contexts (such as the define method on the Class class).
 *
 * @param pgmname  The name of the method we're creating.
 * @param source   The method source (either a string or an array).
 * @param position The position used for reporting errors.  This is the position
 *                 of the source argument for the calling method context.
 * @param parentScope
 *                 The parent code we inherit routine scope from.  This overrides
 *                 anything that might be defined in single method code.
 *
 * @return The constructed method object.
 */
MethodClass *MethodClass::newMethodObject(RexxString *pgmname, RexxObject *source, RexxObject *position, RexxSource *parentSource)
{
    RexxArray *newSourceArray = OREF_NULL;

    // if this is a string object, then convert to a a single element array.
    if (isString(source))
    {
        newSourceArray = new_array((RexxString *)source);
    }
    else
    {
        // request this as an array.  If not convertable, then we'll use it as a string
        RexxArray *newSourceArray = source->requestArray();
        // couldn't convert?
        if (newSourceArray == (RexxArray *)TheNilObject)
        {
            // get the string representation
            RexxString *sourceString = source->makeString();
            // still can't convert?  This is an error
            if (sourceString == (RexxString *)TheNilObject)
            {
                reportException(Error_Incorrect_method_no_method, position);
            }
            // wrap an array around the value
            newSourceArray = new_array(sourceString);
        }
        // have an array of strings (hopefully)
        else
        {
            // must be single dimension
            if (newSourceArray->getDimension() != 1)
            {
                reportException(Error_Incorrect_method_noarray, position);
            }
            // now run through the array make sure we have all string objects.
            ProtectedObject p(newSourceArray);
            for (size_t counter = 1; counter <= newSourceArray->size(); counter++)
            {
                RexxString *sourceString = newSourceArray ->get(counter)->makeString();
                // if this did not convert, this is an error
                if (sourceString == (RexxString *)TheNilObject)
                {
                    reportException(Error_Incorrect_method_nostring_inarray, IntegerTwo);
                }
                else
                {
                    // replace the original item in the array
                    newSourceArray ->put(sourceString, counter);
                }
            }
        }
    }

    MethodClass *result = LanguageParser::createMethod(pgmname, newSourceArray);
    ProtectedObject p(result);

    // if we've been provided with a scope, use it
    if (parentSource == OREF_NULL)
    {
        // see if we have an active context and use the current source as the basis for the lookup
        RexxActivation *currentContext = ActivityManager::currentActivity->getCurrentRexxFrame();
        if (currentContext != OREF_NULL)
        {
            parentSource = currentContext->getSourceObject();
        }
    }

    // if there is a parent source, then merge in the scope information
    if (parentSource != OREF_NULL)
    {
        result->getSourceObject()->inheritSourceContext(parentSource);
    }

    return result;
}


/**
 * Create a new method from REXX code contained in a string or an
 * array
 *
 * @param init_args The array of arguments to the new method.
 * @param argCount  The argument count.
 *
 * @return A new method object.
 */
MethodClass *MethodClass::newRexx(RexxObject **init_args, size_t argCount)
{
    // this method is defined as an instance method, but this is actually attached
    // to a class object instance.  Therefore, any use of the this pointer
    // will be touching the wrong data.  Use the classThis pointer for calling
    // any methods on this object from this method.
    RexxClass *classThis = (RexxClass *)this;

    RexxObject *pgmname;                 // method name
    RexxObject *_source;                 // Array or string object
    MethodClass *newMethod;               // newly created method object
    RexxObject *option = OREF_NULL;
    size_t initCount = 0;                // count of arguments we pass along

    RexxClass::processNewArgs(init_args, argCount, &init_args, &initCount, 2, (RexxObject **)&pgmname, (RexxObject **)&_source);
    // get the method name as a string
    RexxString *nameString = stringArgument(pgmname, ARG_ONE);
    // make sure there is something for the second arge.
    requiredArgument(_source, ARG_TWO);

    RexxSource *sourceContext = OREF_NULL;
    // retrieve extra parameter if exists
    if (initCount != 0)
    {
        RexxClass::processNewArgs(init_args, initCount, &init_args, &initCount, 1, (RexxObject **)&option, NULL);
        if (isOfClass(Method, option))
        {
            sourceContext = ((MethodClass *)option)->getSourceObject();
        }
        else if (isOfClass(Routine, option))
        {
            sourceContext = ((RoutineClass *)option)->getSourceObject();
        }
        else if (isOfClass(Package, option))
        {
            sourceContext = ((PackageClass *)option)->getSourceObject();
        }
        else
        {
            // this must be a string (or convertable) and have a specific value
            option = option->requestString();
            if (option == TheNilObject)
            {
                reportException(Error_Incorrect_method_argType, IntegerThree, "Method, Routine, Package, or String object");
            }
            // default given? set option to NULL (see code below)
            if (!((RexxString *)option)->strCaselessCompare("PROGRAMSCOPE"))
            {
                reportException(Error_Incorrect_call_list, "NEW", IntegerThree, "\"PROGRAMSCOPE\", Method, Routine, Package object", option);
            }
        }
    }
    // go create a method from whatever we were given for source.
    newMethod = newMethodObject(nameString, _source, IntegerTwo, sourceContext);
    ProtectedObject p(newMethod);

    // finish up the object creation.  Set the correct instance behavior (this could
    // be a subclass), check for uninit methods, and finally, send an init message using any
    // left over arguments.
    classThis->completeNewObject(newMethod, init_args, initCount);
    return newMethod;
}


/**
 * Create a method object from a file.
 *
 * @param filename The target file name.
 *
 * @return The created method object.
 */
MethodClass *MethodClass::newFileRexx(RexxString *filename)
{
    // this class is defined on the object class, but this is actually attached
    // to a class object instance.  Therefore, any use of the this pointer
    // will be touching the wrong data.  Use the classThis pointer for calling
    // any methods on this object from this method.
    RexxClass *classThis = (RexxClass *)this;
    // get the method name as a string
    filename = stringArgument(filename, ARG_ONE);

    MethodClass *newMethod = LanguageParser::createMethodFromFile(filename);
    ProtectedObject p(newMethod);

    newMethod->setScope((RexxClass *)TheNilObject);
    classThis->completeNewObject(newMethod);
    return newMethod;
}


/**
 * Create a method from an external library source.
 *
 * @param name   The method name.
 *
 * @return The resolved method object, or OREF_NULL if unable to
 *         load the routine.
 */
MethodClass *MethodClass::loadExternalMethod(RexxString *name, RexxString *descriptor)
{
    name = stringArgument(name, "name");
    descriptor = stringArgument(descriptor, "descriptor");
    // convert external into words
    RexxArray *_words = StringUtil::words(descriptor->getStringData(), descriptor->getLength());
    ProtectedObject p(_words);
    // "LIBRARY libbar [foo]"
    if (((RexxString *)(_words->get(1)))->strCompare(CHAR_LIBRARY))
    {
        RexxString *library = OREF_NULL;
        // the default entry point name is the internal name
        RexxString *entry = name;

        // full library with entry name version?
        if (_words->size() == 3)
        {
            library = (RexxString *)_words->get(2);
            entry = (RexxString *)_words->get(3);
        }
        else if (_words->size() == 2)
        {
            library = (RexxString *)_words->get(2);
        }
        else  // wrong number of tokens
        {
            /* this is an error                  */
            reportException(Error_Translation_bad_external, descriptor);
        }
        // get the native code for this library
        RexxNativeCode *nmethod = PackageManager::loadMethod(library, entry);
        // raise an exception if this entry point is not found.
        if (nmethod == OREF_NULL)
        {
            return (MethodClass *)TheNilObject;
        }
        // turn into a real method object
        return new MethodClass(name, nmethod);
    }
    else
    {
        // unknown external type
        reportException(Error_Translation_bad_external, descriptor);
    }
    return OREF_NULL;
}

