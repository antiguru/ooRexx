/* An environment symbol read once the running package has a local entry:
   one ~local write, then 2,000,000 reads of .stdout, which the search reaches
   after the package local misses. */
.context~package~local~put(1, 'PKGVALUE')
n = 2000000
do i = 1 to n
    x = .stdout
end
say .pkgvalue n
