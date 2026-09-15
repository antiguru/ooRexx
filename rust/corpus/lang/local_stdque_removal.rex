/* Removing .local's STDQUE entry without reading it: setEntry with no value
 * and setMethod both take the entry out of the contents and answer nothing,
 * so neither observes the queue it held.  Afterwards .stdque is the name's
 * own text, a put restores the entry in its old place, and a method under the
 * name answers in its place.
 */

l = .local
l~setEntry('stdque')
say 'setEntry' l~items l~hasIndex('STDQUE') l~allItems~items
say 'indexes ' l~allIndexes~makeString('L', ' ')
say 'symbol  ' .stdque
l['STDQUE'] = 'restored'
say 'restored' l~allIndexes~makeString('L', ' ')
l~setMethod('stdque', 'return "from a method"')
say 'method  ' l~items l['STDQUE'] .stdque l~allIndexes~lastItem
l~unsetMethod('STDQUE')
say 'unset   ' l~items l~hasIndex('STDQUE') .stdque
