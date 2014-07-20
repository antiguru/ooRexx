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
/* REXX Kernel                                           ClassDirective.cpp   */
/*                                                                            */
/* Primitive Translator Abstract Directive Code                               */
/*                                                                            */
/******************************************************************************/
#include "RexxCore.h"
#include "ClassDirective.hpp"
#include "Clause.hpp"
#include "DirectoryClass.hpp"
#include "TableClass.hpp"
#include "ListClass.hpp"
#include "RexxActivation.hpp"
#include "MethodClass.hpp"


/**
 * Construct a ClassDirective.
 *
 * @param n      The name of the requires target.
 * @param p      The public name of the requires target.
 * @param clause The source file clause containing the directive.
 */
ClassDirective::ClassDirective(RexxString *n, RexxString *p, RexxClause *clause) : RexxDirective(clause, KEYWORD_CLASS)
{
    idName = n;
    publicName = p;
}

/**
 * Normal garbage collecting live mark.
 *
 * @param liveMark The current live object mark.
 */
void ClassDirective::live(size_t liveMark)
{
    // must be first one marked (though normally null)
    memory_mark(nextInstruction);
    memory_mark(publicName);
    memory_mark(idName);
    memory_mark(metaclassName);
    memory_mark(subclassName);
    memory_mark(inheritsClasses);
    memory_mark(instanceMethods);
    memory_mark(classMethods);
    memory_mark(dependencies);
}


/**
 * The generalized object marking routine.
 *
 * @param reason The processing faze we're running the mark on.
 */
void ClassDirective::liveGeneral(MarkReason reason)
{
    // must be first one marked (though normally null)
    memory_mark_general(nextInstruction);
    memory_mark_general(publicName);
    memory_mark_general(idName);
    memory_mark_general(metaclassName);
    memory_mark_general(subclassName);
    memory_mark_general(inheritsClasses);
    memory_mark_general(instanceMethods);
    memory_mark_general(classMethods);
    memory_mark_general(dependencies);
}


/**
 * Flatten the directive instance.
 *
 * @param envelope The envelope we're flattening into.
 */
void ClassDirective::flatten(Envelope *envelope)
{
    setUpFlatten(ClassDirective)

        flattenRef(nextInstruction);
        flattenRef(publicName);
        flattenRef(idName);
        flattenRef(metaclassName);
        flattenRef(subclassName);
        flattenRef(inheritsClasses);
        flattenRef(instanceMethods);
        flattenRef(classMethods);
        // we don't carry this one forward
        newThis->dependencies = OREF_NULL;

    cleanUpFlatten
}


/**
 * Allocate a new requires directive.
 *
 * @param size   The size of the object.
 *
 * @return The memory for the new object.
 */
void *ClassDirective::operator new(size_t size)
{
    return new_object(size, T_ClassDirective);
}


/**
 * Do install-time processing of the ::requires directive.  This
 * will resolve the directive and merge all of the public information
 * from the resolved file into this program context.
 *
 * @param activation The activation we're running under for the install.
 */
RexxClass *ClassDirective::install(RexxSource *source, RexxActivation *activation)
{
    RexxClass *metaclass = OREF_NULL;
    RexxClass *subclass = TheObjectClass;

    // make this the current line for the error context
    activation->setCurrent(this);

    if (metaclassName != OREF_NULL)
    {
        // resolve the class.. This must be locatable in the
        // context of the package.
        metaclass = source->findClass(metaclassName);
        if (metaclass == OREF_NULL)
        {
            reportException(Error_Execution_nometaclass, metaclassName);
        }
    }

    // do we have an explicit subclass
    if (subclassName != OREF_NULL)
    {
        // resolve the explicit subclass
        subclass = source->findClass(subclassName);
        if (subclass == OREF_NULL)
        {
            reportException(Error_Execution_noclass, subclassName);
        }
    }

    RexxClass *classObject;       // the class object we're creating

    // create the class object using the appropriate mechanism

    // creating a mixing
    if (mixinClass)
    {
        classObject = subclass->mixinclass(source, idName, metaclass, classMethods);
    }
    // creating a direct subclass
    else
    {
        classObject = subclass->subclass(source, idName, metaclass, classMethods);
    }
    // add the class to the directory, which also protects it from GC for the
    // subsequent steps
    source->addInstalledClass(publicName, classObject, publicClass);

    // using multiple inheritance?  Process each one in turn.
    if (inheritsClasses != OREF_NULL)
    {
        // now handle the multiple inheritance issues
        size_t count = inheritsClasses->items();

        for (size_t i = 1; i <= count; i++)
        {
            // get the next mixin class name and resolve...again, this
            // is required.
            RexxString *inheritsName = (RexxString *)inheritsClasses->get(i);
            RexxClass *mixin = source->findClass(inheritsName);
            if (mixin == OREF_NULL)
            {
                reportException(Error_Execution_noclass, inheritsName);
            }

            // inherit from the mixin
            classObject->sendMessage(OREF_INHERIT, mixin);
        }
    }

    // have instance methods to add?  have the class define them
    if (instanceMethods != OREF_NULL)
    {
        classObject->defineMethods(instanceMethods);
    }

    // the source needs this at the end so it call call the activate methods
    return classObject;
}


/**
 * Check if a dependency this class has is based off of a
 * class co-located in the same class package.
 *
 * @param name   The class name.
 * @param classDirectives
 *               The global local classes list.
 */
void ClassDirective::checkDependency(RexxString *name, DirectoryClass *classDirectives)
{
    if (name != OREF_NULL)
    {
        // if this is in install?
        if (classDirectives->entry(name) != OREF_NULL)
        {
            if (dependencies == OREF_NULL)
            {
                dependencies = new_directory();
            }
            // add to our pending list
            dependencies->setEntry(name, name);
        }
    }
}


/**
 * Check our class dependencies against the locally defined class
 * list to develop a cross dependency list.
 *
 * @param classDirectives
 *               The global set of defined classes in this package.
 */
void ClassDirective::addDependencies(DirectoryClass *classDirectives)
{
    // now for each of our dependent classes, if this is defined locally, we
    // add an entry to our dependency list to aid the class ordering

    checkDependency(metaclassName, classDirectives);
    checkDependency(subclassName, classDirectives);
    // process each inherits item the same way
    if (inheritsClasses != OREF_NULL)
    {
        size_t count = inheritsClasses->items();

        for (size_t i = 1; i <= count; i++)
        {
            RexxString *inheritsName = (RexxString *)inheritsClasses->get(i);
            checkDependency(inheritsName, classDirectives);
        }
    }
}


/**
 * Check if this class has any additional in-package dependencies.
 *
 * @return true if all in-package dependencies have been resolved already.
 */
bool ClassDirective::dependenciesResolved()
{
    return dependencies == OREF_NULL;
}


/**
 * Remove a class from the dependency list.
 *
 * @param name   The name of the class that's next in the ordering.
 */
void ClassDirective::removeDependency(RexxString *name)
{
    // if we have a dependency list, remove this name from the
    // list.  If this is our last dependency item, we can junk
    // the list entirely.
    if (dependencies != OREF_NULL)
    {
        dependencies->remove(name);
        if (dependencies->items() == 0)
        {
            dependencies = OREF_NULL;
        }
    }
}


/**
 * Add an inherits class to the class definition.
 *
 * @param name   The name of the inherited class.
 */
void ClassDirective::addInherits(RexxString *name)
{
    if (inheritsClasses == OREF_NULL)
    {
        inheritsClasses = new_array();
    }
    inheritsClasses->append(name);
}


/**
 * Retrieve the class methods directory for this class.
 *
 * @return The class methods directory.
 */
TableClass *ClassDirective::getClassMethods()
{
    if (classMethods == OREF_NULL)
    {
        classMethods = new_table();
    }
    return classMethods;
}


/**
 * Retrieve the instance methods directory for this class.
 *
 * @return The instance methods directory.
 */
TableClass *ClassDirective::getInstanceMethods()
{
    if (instanceMethods == OREF_NULL)
    {
        instanceMethods = new_table();
    }
    return instanceMethods;
}


/**
 * Check for a duplicate method defined on this class.
 *
 * @param name   The method name.
 * @param classMethod
 *               Indicates whether we are checking for a class or instance method.
 *
 * @return true if this is a duplicate of the method type.
 */
bool ClassDirective::checkDuplicateMethod(RexxString *name, bool classMethod)
{
    if (classMethod)
    {
        return getClassMethods()->get(name) != OREF_NULL;
    }
    else
    {
        return getInstanceMethods()->get(name) != OREF_NULL;
    }

}


/**
 * Add a method to a class definition.
 *
 * @param name   The name to add.
 * @param method The method object that maps to this name.
 * @param classMethod
 *               Indicates whether this is a new class or instance method.
 */
void ClassDirective::addMethod(RexxString *name, MethodClass *method, bool classMethod)
{
    if (classMethod)
    {
        getClassMethods()->put(method, name);
    }
    else
    {
        getInstanceMethods()->put(method, name);
    }
}


/**
 * Add a method to a class definition.
 *
 * @param name   The name to add.
 * @param method The method object that maps to this name.
 */
void ClassDirective::addConstantMethod(RexxString *name, MethodClass *method)
{
    // this gets added as both a class and instance method
    addMethod(name, method, false);
    addMethod(name, method, true);
}
