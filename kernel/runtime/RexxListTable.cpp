/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 1995, 2004 IBM Corporation. All rights reserved.             */
/* Copyright (c) 2005-2006 Rexx Language Association. All rights reserved.    */
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
/* REXX Kernel                                             RexxListTable.cpp  */
/*                                                                            */
/* Primitive List Table Class                                                 */
/*                                                                            */
/******************************************************************************/
#include "RexxCore.h"
#include "ListClass.hpp"

void RexxListTable::live(size_t liveMark)
/******************************************************************************/
/* Function:  Normal garbage collection live marking                          */
/******************************************************************************/
{
  size_t     index;                    /* working index                     */
                                       /* loop through our table            */
  for (index = 0; index < this->size; index++)
  {
                                       /* mark an element                   */
      memory_mark(this->elements[index].value);
  }
}

void RexxListTable::liveGeneral(int reason)
/******************************************************************************/
/* Function:  Generalized object marking                                      */
/******************************************************************************/
{
  size_t     index;                    /* working index                     */

                                       /* loop through our table            */
  for (index = 0; index < this->size; index++)
  {
                                       /* mark an element                   */
      memory_mark_general(this->elements[index].value);
  }
}

void   RexxListTable::flatten(RexxEnvelope *envelope)
/******************************************************************************/
/* Function:  Flatten an object                                               */
/******************************************************************************/
{
 setUpFlatten(RexxListTable)

 size_t i;

   for (i = this->size - 1; i >= 0 ; i--)
   {
       flatten_reference(newThis->elements[i].value, envelope);
   }

 cleanUpFlatten
}


void *RexxListTable::operator new(size_t size, size_t initialSize)
/******************************************************************************/
/* Function:  Construct and initialized a new list item                       */
/******************************************************************************/
{
  RexxListTable *newTable;             /* newly created list                */

                                       /* Get new object                    */
  newTable = (RexxListTable *)new_object(size + sizeof(LISTENTRY) * (initialSize - 1));
                                       /* Give new object its behaviour     */
  newTable->setBehaviour(TheListTableBehaviour);
  newTable->clearObject();
  newTable->size = initialSize;
  return newTable;                     /* return the new list item          */
}


void *RexxListTable::operator new(size_t size, size_t initialSize, size_t companionSize)
/******************************************************************************/
/* Function:  Construct and initialized a new list item                       */
/******************************************************************************/
{
  RexxListTable *newTable;             /* newly created list                */
  RexxList      *newList;              /* associated list object            */
  size_t bytes;                        /* size of the allocated object      */

                                       /* Compute size of hash tab object   */
  bytes = roundObjectBoundary(size + (sizeof(LISTENTRY) * (initialSize - 1)));
  /* make sure we've got proper sizes for each of the object parts. */
  companionSize = roundObjectBoundary(companionSize);
                                       /* Get space for two objects         */
                                       /* Get new object                    */
  newList  = (RexxList *)new_object(bytes + companionSize);
  newList->clearObject();              /* clear the entire lot              */
                                       /* address the list table            */
  newTable = (RexxListTable *)(((char *)newList) + companionSize);
                                       /* compute total size of the list    */
                                       /* table (allowing for possible      */
                                       /* over allocation by the memory     */
                                       /* manager                           */
  bytes = newList->getObjectSize() - companionSize;

  // initialize the hash table object
  ((RexxObject *)newTable)->initializeNewObject(bytes, memoryObject.markWord, RexxMemory::virtualFunctionTable[T_ListTable], TheListTableBehaviour);
                                       /* reduce the companion size         */
  newList->setObjectSize(companionSize);
  newTable->size = initialSize;        /* fill in the initial size          */
                                       /* hook the list into the companion  */
                                       /* OrefSet is not used, because the  */
                                       /* companion object is not fully set */
  newList->table = newTable;           /* up yet (no behaviour)             */
  return newList;                      /* return the new list item          */
}

