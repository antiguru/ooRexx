/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 1995, 2004 IBM Corporation. All rights reserved.             */
/* Copyright (c) 2005-2014 Rexx Language Association. All rights reserved.    */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* http://www.oorexx.org/license.html                          */
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
/* REXX Kernel                                                RexxMemory.hpp  */
/*                                                                            */
/* Primitive Memory Class Definitions                                         */
/*                                                                            */
/******************************************************************************/

#ifndef Included_RexxMemory
#define Included_RexxMemory

#include "SysSemaphore.hpp"
#include "IdentityTableClass.hpp"

// this can be enabled to switch on memory profiling info
//#define MEMPROFILE

#ifdef __REXX64__
// The minimum allocation unit for an object.
// 16 is needed for 64-bit to maintain some required alignments
#define ObjectGrain 16
/* The unit of granularity for large allocation */
#define LargeAllocationUnit 2048
/* The unit of granularity for extremely large objects */
#define VeryLargeAllocationUnit 8192
/* Minimum size of an object.  This is not the actual minimum size, */
/* but we allocate objects with an 8-byte granularity */
/* this is the granularity for objects greater than 16Mb. */
#define VeryLargeObjectGrain    512

/* Minimum size of an object.  This is not the actual minimum size, */
/* but we allocate objects with a defined granularity */
/* This is the smallest object we'll allocate from storage.  */
#define MinimumObjectSize ((size_t)48)
#define MaximumObjectSize ((size_t)0xfffffffffffffff0ull)
#else
/* The minimum allocation unit for an object.   */
#define ObjectGrain 8
/* The unit of granularity for large allocation */
#define LargeAllocationUnit 1024
/* The unit of granularity for extremely large objects */
#define VeryLargeAllocationUnit 4096
/* this is the granularity for objects greater than 16Mb. */
#define VeryLargeObjectGrain    256

/* Minimum size of an object.  This is not the actual minimum size, */
/* but we allocate objects with an 8-byte granularity */
/* This is the smallest object we'll allocate from storage.  */
#define MinimumObjectSize ((size_t)24)
#define MaximumObjectSize ((size_t)0xfffffff0)
#endif

inline void SetObjectLive(void *o, size_t mark) {
    ((RexxObject *)o)->setObjectLive(mark);
}
#define IsObjectGrained(o)  ((((size_t)o)%ObjectGrain) == 0)
#define IsValidSize(s) ((s) >= MinimumObjectSize && ((s) % ObjectGrain) == 0)

inline size_t roundObjectBoundary(size_t n) { return RXROUNDUP(n,ObjectGrain); }
inline size_t roundLargeObjectAllocation(size_t n) { return RXROUNDUP(n, LargeAllocationUnit); }
inline size_t roundObjectResize(size_t n) { return RXROUNDUP(n, ObjectGrain); }

class RexxActivationFrameBuffer;
class MemorySegment;
class MemorySegmentPool;
class MethodClass;
class RexxVariable;
class WeakReference;
class RexxIdentityTable;
class GlobalProtectedObject;

#ifdef _DEBUG
class RexxMemory;
#endif


enum
{
    LIVEMARK,
    RESTORINGIMAGE,
    SAVINGIMAGE,
    FLATTENINGOBJECT,
    UNFLATTENINGOBJECT,
};


typedef char MEMORY_POOL_STATE;

class MemorySegmentPoolHeader
{
#ifdef _DEBUG
 friend class RexxMemory;
#endif

 protected:
     MemorySegmentPool *next;
     MemorySegment     *spareSegment;
     char  *nextAlloc;
     char  *nextLargeAlloc;
     size_t uncommitted;
     size_t reserved;            // force aligment of the state data....
};

class MemorySegmentPool : public MemorySegmentPoolHeader
{
#ifdef _DEBUG
 friend class RexxMemory;
#endif
 friend bool SysAccessPool(MemorySegmentPool **);
 public:
     void          *operator new(size_t size, size_t minSize);
     inline void   *operator new(size_t size, void *pool) { return pool;}
     inline void    operator delete(void *, size_t) { }
     inline void    operator delete(void *, void *) { }

     static MemorySegmentPool *createPool();

     MemorySegmentPool();
     MemorySegment *newSegment(size_t minSize);
     MemorySegment *newLargeSegment(size_t minSize);
     void               freePool(void);
     MemorySegmentPool *nextPool() {return this->next;}
     void               setNext( MemorySegmentPool *nextPool ); /* CHM - def.96: new function */

 private:
     char           state[8];    // must be at the end of the structure.
};

#include "MemoryStats.hpp"
#include "MemorySegment.hpp"

class RexxMemory : public RexxInternalObject
{
#ifdef _DEBUG
  friend class RexxInstructionOptions;
#endif
 public:
    inline RexxMemory();
    inline RexxMemory(RESTORETYPE restoreType) { ; };

    inline operator RexxObject*() { return (RexxObject *)this; };
    inline RexxObject *operator=(DeadObject *d) { return (RexxObject *)this; };

    void live(size_t);
    void liveGeneral(int reason);
    void flatten(RexxEnvelope *);
    RexxObject  *makeProxy(RexxEnvelope *);

    void        initialize(bool restoringImage);
    MemorySegment *newSegment(size_t requestLength, size_t minLength);
    MemorySegment *newLargeSegment(size_t requestLength, size_t minLength);
    RexxObject *oldObject(size_t size);
    inline RexxObject *newObject(size_t size) { return newObject(size, T_Object); }
    RexxObject *newObject(size_t size, size_t type);
    RexxObject *temporaryObject(size_t size);
    RexxArray  *newObjects(size_t size, size_t count, size_t objectType);
    void        reSize(RexxObject *, size_t);
    void        checkUninit();
    void        runUninits();
    void        removeUninitObject(RexxObject *obj);
    void        addUninitObject(RexxObject *obj);
    bool        isPendingUninit(RexxObject *obj);
    inline void checkUninitQueue() { if (pendingUninits > 0) runUninits(); }

    void        markObjects(void);
    void        markObjectsMain(RexxObject *);
    void        killOrphans(RexxObject *);
    void        mark(RexxObject *);
    void        markGeneral(void *);
    void        collect();
    inline RexxObject *saveObject(RexxInternalObject *saveObj) {this->saveTable->add((RexxObject *)saveObj, (RexxObject *)saveObj); return (RexxObject *)saveObj;}
    inline void        discardObject(RexxInternalObject *obj) {this->saveTable->remove((RexxObject *)obj);};
    inline void        removeHold(RexxInternalObject *obj) { this->saveStack->remove((RexxObject *)obj); }
    void        discardHoldObject(RexxInternalObject *obj);
    RexxObject *holdObject(RexxInternalObject *obj);
    void        saveImage();
    bool        savingImage() { return saveimage; }
    bool        restoringImage() { return restoreimage; }
    RexxObject *setDump(RexxObject *);
    inline bool queryDump() {return this->dumpEnable;};
    RexxObject *dump();
    void        dumpObject(RexxObject *objectRef, FILE *outfile);
    void        setObjectOffset(size_t offset);
    void        setEnvelope(RexxEnvelope *);
    inline void        setMarkTable(RexxTable *marktable) {this->markTable = marktable;};
    inline void        setOrphanCheck(bool orphancheck) {this->orphanCheck = orphancheck; };
    RexxObject *checkSetOref(RexxObject *, RexxObject **, RexxObject *, const char *, int);
    RexxObject *setOref(void *index, RexxObject *value);
    RexxStack  *getFlattenStack();
    void        returnFlattenStack();
    RexxObject *reclaim();
    RexxObject *setParms(RexxObject *, RexxObject *);
    RexxObject *gutCheck();
    void        memoryPoolAdded(MemorySegmentPool *);
    void        shutdown();
    void        liveStackFull();
    void        dumpMemoryProfile();
    char *      allocateImageBuffer(size_t size);
    void        logVerboseOutput(const char *message, void *sub1, void *sub2);
    inline void verboseMessage(const char *message) {
  #ifdef VERBOSE_GC
        logVerboseOutput(message, NULL, NULL);
  #endif
    }

    inline void verboseMessage(const char *message, size_t sub1) {
  #ifdef VERBOSE_GC
        logVerboseOutput(message, (void *)sub1, NULL);
  #endif
    }

    inline void verboseMessage(const char *message, size_t sub1, size_t sub2) {
  #ifdef VERBOSE_GC
        logVerboseOutput(message, (void *)sub1, (void *)sub2);
  #endif
    }

    inline void logObjectStats(RexxObject *obj) { imageStats->logObject(obj); }
    inline void pushSaveStack(RexxObject *obj) { saveStack->push(obj); }
    inline void removeSavedObject(RexxObject *obj) { saveStack->remove(obj); }
    inline void disableOrefChecks() { checkSetOK = false; }
    inline void enableOrefChecks() { checkSetOK = true; }
    inline void clearSaveStack() { saveStack->clear(); }

    void        checkAllocs();
    RexxObject *dumpImageStats();
    static void createLocks();
    static void closeLocks();
    void        scavengeSegmentSets(MemorySegmentSet *requester, size_t allocationLength);
    void        setUpMemoryTables(RexxIdentityTable *old2newTable);
    void        collectAndUninit(bool clearStack);
    void        lastChanceUninit();
    inline RexxDirectory *getGlobalStrings() { return globalStrings; }
    void        addWeakReference(WeakReference *ref);
    void        checkWeakReferences();

    static void restore();
    static void buildVirtualFunctionTable();
    static void create();
    static void createImage();
    static RexxString *getGlobalName(const char *value);
    static void createStrings();
    static RexxArray *saveStrings();
    static void restoreStrings(RexxArray *stringArray);

    static void *virtualFunctionTable[];             /* table of virtual functions        */
    static PCPPM exportedMethods[];      /* start of exported methods table   */

    size_t markWord;                     /* current marking counter           */
    int    markReason;                   // reason for calling liveGeneral()
    RexxVariable *variableCache;         /* our cache of variable objects     */
    GlobalProtectedObject *protectedObjects;  // specially protected objects

    static RexxDirectory *environment;      // global environment
    static RexxDirectory *functionsDir;     // statically defined requires
    static RexxDirectory *commonRetrievers; // statically defined requires
    static RexxDirectory *kernel;           // the kernel directory
    static RexxDirectory *system;           // the system directory

private:

    // default size for the live stack (in entries)
    static const size_t LiveStackSize = 32 * 1024;
    // the number of newly created items to stack in the save stack
    static const size_t SaveStackSize = 10;
    // the pre-allocation size for the save stack
    static const size_t SaveStackAllocSize = 500;
    // the maximum size for the startup image size
    static const size_t MaxImageSize = 2000000;

/******************************************************************************/
/* Define location of objects saved in SaveArray during Saveimage processing  */
/*  and used during restart processing.                                       */
/* Currently only used in OKMEMORY.C                                          */
/******************************************************************************/
enum
{
    saveArray_ENV = 1,
    saveArray_KERNEL,
    saveArray_NAME_STRINGS,
    saveArray_TRUE,
    saveArray_FALSE,
    saveArray_NIL,
    saveArray_GLOBAL_STRINGS,
    saveArray_CLASS,
    saveArray_PBEHAV,
    saveArray_PACKAGES,
    saveArray_NULLA,
    saveArray_NULLPOINTER,
    saveArray_SYSTEM,
    saveArray_FUNCTIONS,
    saveArray_COMMON_RETRIEVERS,
    saveArray_highest = saveArray_COMMON_RETRIEVERS
};


    inline void checkLiveStack() { if (!liveStack->checkRoom()) liveStackFull(); }
    inline void pushLiveStack(RexxObject *obj) { checkLiveStack(); liveStack->fastPush(obj); }
    inline RexxObject * popLiveStack() { return (RexxObject *)liveStack->fastPop(); }
    inline void bumpMarkWord() { markWord ^= MarkMask; }
    inline void restoreMark(RexxObject *markObject, RexxObject **pMarkObject) {
                                         /* we update the object's location   */
        *pMarkObject = (RexxObject *)((size_t)markObject + relocation);
    }

    inline void unflattenMark(RexxObject *markObject, RexxObject **pMarkObject) {
                                         /* do the unflatten                  */
        *pMarkObject = markObject->unflatten(this->envelope);
    }

    inline void restoreObjectMark(RexxObject *markObject, RexxObject **pMarkObject) {
                                           /* update the object reference       */
        markObject = (RexxObject *)((char *)markObject + objOffset);
        markObject->setObjectLive(markWord); /* Then Mark this object as live.    */
        *pMarkObject = markObject;         /* now set this back again           */
    }


  /* object validation method --used to find and diagnose broken object references       */
    void saveImageMark(RexxObject *markObject, RexxObject **pMarkObject);
    void orphanCheckMark(RexxObject *markObject, RexxObject **pMarkObject);

    bool inObjectStorage(RexxObject *obj);
    bool inSharedObjectStorage(RexxObject *obj);
    bool objectReferenceOK(RexxObject *o);
    void restoreImage();

    static void defineKernelMethod(const char *name, RexxBehaviour * behaviour, PCPPM entryPoint, size_t arguments);
    static void defineProtectedKernelMethod(const char *name, RexxBehaviour * behaviour, PCPPM entryPoint, size_t arguments);
    static void definePrivateKernelMethod(const char *name, RexxBehaviour * behaviour, PCPPM entryPoint, size_t arguments);

    RexxStack  *liveStack;
    RexxStack  *flattenStack;
    RexxSaveStack      *saveStack;
    RexxIdentityTable  *saveTable;
    RexxTable  *markTable;               /* tabobjects to start a memory mark */
                                         /*  if building/restoring image,     */
                                         /*OREF_ENV, else old2new             */
    RexxIdentityTable  *old2new;           /* remd set                          */
    RexxIdentityTable  *uninitTable;       // the table of objects with uninit methods
    size_t            pendingUninits;    // objects waiting to have uninits run
    bool              processingUninits; // true when we are processing the uninit table

    MemorySegmentPool *firstPool;        /* First segmentPool block.          */
    MemorySegmentPool *currentPool;      /* Curent segmentPool being carved   */
    OldSpaceSegmentSet oldSpaceSegments;
    NormalSegmentSet newSpaceNormalSegments;
    LargeSegmentSet  newSpaceLargeSegments;
    char *image_buffer;                  /* the buffer used for image save/restore operations */
    size_t image_offset;                 /* the offset information for the image */
    size_t relocation;                   /* image save/restore relocation factor */
    bool dumpEnable;                     /* enabled for dumps?                */
    bool saveimage;                      /* we're saving the image */
    bool restoreimage;                   /* we're restoring the image */
    bool checkSetOK;                     /* OREF checking is enabled          */
                                         /* enabled for checking for bad      */
                                         /*OREF's?                            */
    bool orphanCheck;
    size_t objOffset;                    /* offset of arriving mobile objects */
                                         /* envelope for arriving mobile      */
                                         /*objects                            */
    RexxEnvelope *envelope;
    RexxStack *originalLiveStack;        /* original live stack allocation    */
    MemoryStats *imageStats;             /* current statistics collector      */

    size_t allocations;                  /* number of allocations since last GC */
    size_t collections;                  /* number of garbage collections     */
    WeakReference *weakReferenceList;    // list of active weak references

    static RexxDirectory *globalStrings; // table of global strings
    static SysMutex flattenMutex;        /* locks for various memory processes */
    static SysMutex unflattenMutex;
    static SysMutex envelopeMutex;
};


/******************************************************************************/
/* Memory management macros                                                   */
/******************************************************************************/


inline void saveObject(RexxInternalObject *o) { memoryObject.saveObject((RexxObject *)o); }
inline void discardObject(RexxInternalObject *o) { memoryObject.discardObject((RexxObject *)o); }
inline void holdObject(RexxInternalObject *o) { memoryObject.holdObject((RexxObject *)o); }
inline void discardHoldObject(RexxInternalObject *o) { memoryObject.discardHoldObject((RexxObject *)(o)); }


inline RexxObject *new_object(size_t s) { return memoryObject.newObject(s); }
inline RexxObject *new_object(size_t s, size_t t) { return memoryObject.newObject(s, t); }

inline RexxArray *new_arrayOfObject(size_t s, size_t c, size_t t)  { return memoryObject.newObjects(s, c, t); }


// memory marking macros
#define ObjectNeedsMarking(oref) ((oref) != OREF_NULL && !((oref)->isObjectMarked(liveMark)) )
#define memory_mark(oref)  if (ObjectNeedsMarking(oref)) memoryObject.mark((RexxObject *)(oref))
#define memory_mark_general(oref) (memoryObject.markGeneral((void *)&(oref)))

// some convenience macros for marking arrays of objects.
#define memory_mark_array(count, array) \
  for (size_t i = 0; i < count; i++)    \
  {                                     \
      memory_mark(array[i]);            \
  }

#define memory_mark_general_array(count, array) \
  for (size_t i = 0; i < count; i++)            \
  {                                             \
      memory_mark_general(array[i]);            \
  }

// Following macros are for Flattening and unflattening of objects
// Some notes on what is going on here.  The flatten() method gets called on an object
// after it has been moved into the envelope buffer, so the this pointer is
// to the copied object, not the original.  On a call to flattenReference(), it might
// be necessary to allocate a larger buffer.  When that happens, the copied object gets
// moved to a new location and the newThis pointer gets updated to the new object location.
// it is necessary to copy the this pointer and also declare the newThis pointer as volatile
// so that the change in pointer value doesn't get optimized out by the compiler.

// set up for flattening.  This sets up the newThis pointer and also gets some
// information from the envelope.  The type argument allows newThis to be declared with
// the correct type.
#define setUpFlatten(type)        \
  {                               \
  size_t newSelf = envelope->currentOffset; \
  type * volatile newThis = (type *)this;   // NB:  This is declared volatile to avoid optimizer problems.

// just a block closer for the block created by setUpFlatten.
#define cleanUpFlatten                    \
 }

// flatten an individual reference.  This version is deprecated, use flattenRef instead.
#define flatten_reference(oref,envel)  if ((oref) != OREF_NULL) envel->flattenReference((void *)&newThis, newSelf, (void *)&(oref))
// newer, simplified form.  Just give the name of the field.
#define flattenRef(oref)  if ((newThis->oref) != OREF_NULL) envelope->flattenReference((void *)&newThis, newSelf, (void *)&(newThis->oref))

// a version for flattening arrays of objects.  Give the count field and the name of the array.
#define flattenArrayRefs(count, array)          \
  for (size_t i = 0; i < count; i++)            \
  {                                             \
      flattenRef(array[i]);                     \
  }

// declare a class creation routine
// for classes with their own
// explicit class objects
#define CLASS_CREATE(name, id, className) The##name##Class = new className(id, The##name##ClassBehaviour, The##name##Behaviour);

#endif
