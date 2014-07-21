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
/* REXX Kernel                                               StemClass.hpp    */
/*                                                                            */
/* Primitive Stem Class Definitions                                           */
/*                                                                            */
/******************************************************************************/
#ifndef Included_StemClass
#define Included_StemClass

#include "CompoundTableElement.hpp"
#include "CompoundVariableTable.hpp"
#include "ExpressionStem.hpp"

#define SORT_CASESENSITIVE 0
#define SORT_CASEIGNORE    1

#define SORT_ASCENDING 0
#define SORT_DECENDING 1

class SupplierClass;
class DirectoryClass;
class CompoundVariableTail;
class RexxNativeActivation;

 class SortData
 {
 public:
     stringsize_t startColumn;
     stringsize_t columnLength;
 };


class StemClass : public RexxObject
{
  friend class CompoundVariableTable;
  public:
    void *operator new (size_t);
    inline void *operator new(size_t size, void *ptr) {return ptr;};
    StemClass(RexxString *);
    inline StemClass(RESTORETYPE restoreType) { ; };

    virtual void live(size_t);
    virtual void liveGeneral(MarkReason reason);
    virtual void flatten(Envelope*);
    virtual RexxObject * copy();

    void         copyFrom(CompoundVariableTable &_tails);
    bool         numberValue(wholenumber_t &result, size_t precision);
    bool         numberValue(wholenumber_t &result);
    bool         unsignedNumberValue(stringsize_t &result, size_t precision);
    bool         unsignedNumberValue(stringsize_t &result);
    bool         doubleValue(double &result);
    NumberString *numberString();
    RexxInteger *integerValue(size_t);
    RexxString  *stringValue();
    ArrayClass   *makeArray();
    ArrayClass   *allItems();
    ArrayClass   *allIndexes();
    SupplierClass *supplier();
    DirectoryClass *toDirectory();
    RexxObject  *request(RexxString *);
    RexxObject   *empty();
    RexxObject   *isEmptyRexx();
    bool          isEmpty();
    size_t        items();

    void        dropValue();
    RexxObject *getStemValue();
    RexxObject *unknown (RexxString *, ArrayClass *);
    RexxObject *bracket (RexxObject **, size_t);
    RexxObject *bracketEqual(RexxObject **, size_t);

    RexxObject *hasIndex(RexxObject **, size_t);
    RexxObject *remove(RexxObject **, size_t);
    RexxObject *hasItem(RexxObject *);
    RexxObject *index(RexxObject *);
    RexxObject *itemsRexx();
    RexxObject *removeItem(RexxObject *);


    RexxString *tail(ArrayClass *, size_t);
    RexxObject *newRexx(RexxObject **, size_t);
    RexxObject *evaluateCompoundVariableValue(RexxActivation *context, RexxString *stemVariableName, CompoundVariableTail &resolved_tail);
    RexxObject *getCompoundVariableValue(CompoundVariableTail &resolved_tail);
    RexxObject *getCompoundVariableRealValue(CompoundVariableTail &resolved_tail);
    RexxObject *realCompoundVariableValue(CompoundVariableTail &resolved_tail);
    CompoundTableElement *getCompoundVariable(CompoundVariableTail &name);
    CompoundTableElement *exposeCompoundVariable(CompoundVariableTail &name);
    CompoundTableElement *findCompoundVariable(CompoundVariableTail &name);
    CompoundTableElement *findByValue(RexxObject *target);
    void        dropCompoundVariable(CompoundVariableTail &name);
    void        setCompoundVariable(CompoundVariableTail &name, RexxObject *value);
    void        setValue(RexxInternalObject *value);
    ArrayClass  *tailArray();
    CompoundTableElement *nextVariable(RexxNativeActivation *activation);
    RexxObject *handleNovalue(RexxActivation *context, RexxString *name, RexxObject *defaultValue, CompoundTableElement *variable);
    void        expose(CompoundTableElement &variable);
    bool        sort(RexxString *prefix, int order, int type, size_t start, size_t end, size_t firstcol, size_t lastcol);
    void        mergeSort(SortData *sd, int (*comparator)(SortData *, RexxString *, RexxString *), RexxString **strings, RexxString **working, size_t left, size_t right);
    void        merge(SortData *sd, int (*comparator)(SortData *, RexxString *, RexxString *), RexxString **strings, RexxString **working, size_t left, size_t mid, size_t right);
    size_t      find(SortData *sd, int (*comparator)(SortData *, RexxString *, RexxString *), RexxString **strings, RexxString *val, int bnd, size_t left, size_t right);
    void        arraycopy(RexxString **source, size_t start, RexxString **target, size_t index, size_t count);

    inline bool compoundVariableExists(CompoundVariableTail &resolved_tail) { return realCompoundVariableValue(resolved_tail) != OREF_NULL; }
    inline RexxString *getName() { return stemName; }
    inline CompoundTableElement *first() { return tails.first(); }
           RexxString *createCompoundName(CompoundVariableTail &tailPart);
    inline void init() { tails.init(this); }

    void setElement(const char *tail, RexxInternalObject *value);
    void setElement(size_t tail, RexxInternalObject *value);
    void dropElement(const char *tail);
    void dropElement(size_t tail);
    void dropElement(CompoundVariableTail &tail);
    RexxInternalObject *getElement(size_t tail);
    RexxInternalObject *getElement(const char *tail);
    RexxInternalObject *getElement(CompoundVariableTail &tail);

    static void createInstance();
    static RexxClass *classInstance;

 protected:

    RexxString *stemName;               // the name of the stem
    CompoundVariableTable tails;        // the table of compound tails
    RexxInternalObject *value;          // value of the stem
    bool dropped;                       // stem has no explicit value

};
 #endif
