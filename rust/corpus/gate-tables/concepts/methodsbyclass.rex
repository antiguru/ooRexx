/* provide.xml `methodsbyclass`: the objects the following chapters describe
   are generally available through environment symbols, and the [] and []=
   syntax diagrams show the index inside the brackets -- matrix[2, 3] = 0 is
   the []= method of a multidimensional array, and the section states that
   matrix[2, 3] = 0 and matrix~"[]="(0, 2, 3) are both valid and equivalent,
   which the overwrite below reads back through the operator form.

   The `order` line is what lets this row see the indexing at all: a write and
   the read that matches it miss together under any injective index mapping,
   so only a reader of the slots in their own order can tell one mapping from
   another. */
matrix = .array~new(2, 3)
matrix[2, 3] = 0
matrix[1, 2] = 'a12'
matrix~"[]="('a21', 2, 1)
matrix~"[]="('z23', 2, 3)
say 'element' matrix[2, 3] matrix[1, 2] matrix[2, 1]
say 'order' matrix~toString('l', ' ')
say 'dimensions' matrix~dimension matrix~dimension(1) matrix~dimension(2)
say 'methods' matrix~hasMethod("[]") matrix~hasMethod("[]=")
say 'environment' .array~id .directory~id .stringtable~id
