/* provide.xml `methodsbyclass`: the objects the following chapters describe
   are generally available through environment symbols, and the [] and []=
   syntax diagrams show the index inside the brackets -- matrix[2, 3] = 0 is
   the []= method of a multidimensional array. */
matrix = .array~new(2, 3)
matrix[2, 3] = 0
say 'element' matrix[2, 3]
say 'dimensions' matrix~dimension
say 'methods' matrix~hasMethod("[]") matrix~hasMethod("[]=")
say 'environment' .array~id .directory~id .stringtable~id
