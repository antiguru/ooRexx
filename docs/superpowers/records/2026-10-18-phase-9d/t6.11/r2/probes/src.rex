m = .array~method('SORTWITH')
say m~source~items m~isPrivate m~isGuarded m~isProtected m~scope~id
say .array~method('STABLESORTWITH')~source~items
say .array~new~hasMethod('sortwith') .array~instanceMethod('SORTWITH')~class~id
a = .array~of(3,1,2)
say a~sortWith(.numericcomparator~new)~toString(, ' ')
say a~stableSortWith(.descendingcomparator~new)~toString(, ' ')
.array~new(3)~sortWith(.numericcomparator~new)
