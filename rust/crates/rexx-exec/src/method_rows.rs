//! A table of per-method rows indexed by [`MethodId`].

use rexx_classes::MethodId;
use rustc_hash::FxHashMap;

/// Rows keyed by [`MethodId`]: a vector indexed by the id for the identities
/// filed before [`MethodRows::seal`], and a map for those filed after, so an
/// identity minted at run time leaves nothing behind once its row is removed.
pub(crate) struct MethodRows<T> {
    dense: Vec<Option<T>>,
    sealed: bool,
    minted: FxHashMap<MethodId, T>,
}

impl<T: Copy> MethodRows<T> {
    pub(crate) fn new() -> Self {
        MethodRows {
            dense: Vec::new(),
            sealed: false,
            minted: FxHashMap::default(),
        }
    }

    /// Stops the vector growing: every later row past its end goes in the map.
    pub(crate) fn seal(&mut self) {
        self.sealed = true;
    }

    #[inline]
    pub(crate) fn get(&self, method: MethodId) -> Option<T> {
        match self.dense.get(method.0 as usize) {
            Some(row) => *row,
            None if self.minted.is_empty() => None,
            None => self.minted.get(&method).copied(),
        }
    }

    pub(crate) fn get_mut(&mut self, method: MethodId) -> Option<&mut T> {
        match self.dense.get_mut(method.0 as usize) {
            Some(row) => row.as_mut(),
            None => self.minted.get_mut(&method),
        }
    }

    pub(crate) fn insert(&mut self, method: MethodId, row: T) {
        let index = method.0 as usize;
        if index < self.dense.len() {
            self.dense[index] = Some(row);
        } else if self.sealed {
            self.minted.insert(method, row);
        } else {
            self.dense.resize(index + 1, None);
            self.dense[index] = Some(row);
        }
    }

    pub(crate) fn remove(&mut self, method: MethodId) {
        match self.dense.get_mut(method.0 as usize) {
            Some(row) => *row = None,
            None => {
                self.minted.remove(&method);
            }
        }
    }

    /// Drops every row `keep` answers false for.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        for row in &mut self.dense {
            if row.as_ref().is_some_and(|row| !keep(row)) {
                *row = None;
            }
        }
        self.minted.retain(|_, row| keep(row));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.minted.is_empty() && self.dense.iter().all(Option::is_none)
    }

    #[cfg(test)]
    pub(crate) fn rows(&self) -> impl Iterator<Item = &T> {
        self.dense.iter().flatten().chain(self.minted.values())
    }

    /// The vector's length and the map's, for the tests that pin that neither
    /// grows with the identities a program mints and drops.
    #[cfg(test)]
    pub(crate) fn extent(&self) -> (usize, usize) {
        (self.dense.len(), self.minted.len())
    }
}
