use super::MethodRows;
use rexx_classes::MethodId;

/// After the seal, an id the vector covers still lands in the vector,
/// and one past it in the map.
#[test]
fn a_sealed_table_files_by_the_vectors_reach() {
    let mut rows = MethodRows::new();
    rows.insert(MethodId(3), 'a');
    rows.seal();
    rows.insert(MethodId(1), 'b');
    rows.insert(MethodId(9), 'c');
    assert_eq!(rows.extent(), (4, 1));
    assert_eq!(rows.get(MethodId(1)), Some(&'b'));
    rows.remove(MethodId(9));
    assert_eq!(rows.extent(), (4, 0));
}

/// A map row below the vector's end is refused in a debug build.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "sits below the vector's end")]
fn a_minted_row_below_the_vector_is_refused() {
    let mut rows = MethodRows::new();
    rows.insert(MethodId(3), 'a');
    rows.seal();
    rows.minted.insert(MethodId(1), 'b');
    rows.get_mut(MethodId(1));
}
