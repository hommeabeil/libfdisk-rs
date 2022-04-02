//! Container for fdisk partitions. The container does not have any
//! real connection with label (partition table) and with real on-disk data.

use crate::errors::*;
use crate::iter::Iter;
use crate::partition::Partition;
use crate::Context;
use libfdisk_sys;

/// Container for fdisk partitions
pub struct Table {
    pub(crate) ptr: *mut libfdisk_sys::fdisk_table,
}

impl Table {
    /// Return newly allocated table struct
    pub fn new() -> Table {
        Table {
            ptr: unsafe { libfdisk_sys::fdisk_new_table() },
        }
    }

    /// Increments reference counter.
    pub fn ref_table(&self) {
        unsafe { libfdisk_sys::fdisk_ref_table(self.ptr) }
    }

    /// Removes all entries (partitions) from the table. The parititons
    /// with zero reference count will be deallocated.
    /// This function does not modify partition table.
    pub fn reset_table(&self) -> Result<()> {
        match unsafe { libfdisk_sys::fdisk_reset_table(self.ptr) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    ///Adds a new entry to table and increment pa reference counter.
    /// Don't forget to use unref_partition() after table_add_partition()
    /// if you want to keep the pa referenced by the table only.
    /// # Arguments
    /// * `pa` - partition
    pub fn add_partition(&self, pa: &mut Partition) -> Result<()> {
        match unsafe { libfdisk_sys::fdisk_table_add_partition(self.ptr, pa.ptr) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Return number of entries in table
    pub fn nents(&self) -> usize {
        unsafe { libfdisk_sys::fdisk_table_get_nents(self.ptr) }
    }

    /// Return n-th entry from table
    pub fn partition(&self, n: usize) -> Option<Partition> {
        let ptr = unsafe { libfdisk_sys::fdisk_table_get_partition(self.ptr, n) };
        if ptr.is_null() {
            return None;
        }
        Some(Partition { ptr })
    }

    /// Return partition with partno.
    pub fn partition_by_partno(&self, partno: usize) -> Option<Partition> {
        let ptr = unsafe { libfdisk_sys::fdisk_table_get_partition_by_partno(self.ptr, partno) };
        if ptr.is_null() {
            return None;
        }
        Some(Partition { ptr })
    }

    /// Return true if the table is without filesystems
    pub fn is_empty(&self) -> bool {
        unsafe { libfdisk_sys::fdisk_table_is_empty(self.ptr) == 1 }
    }

    /// Removes the pa from the table and de-increment reference counter of the pa .
    /// The partition with zero reference counter will be deallocated. Don't forget
    /// to use ref_partition() before call remove_partition() if you want to use pa later.
    ///
    /// # Arguments
    /// * `pa` - partition
    pub fn remove_partition(&self, pa: &mut Partition) -> Result<()> {
        match unsafe { libfdisk_sys::fdisk_table_remove_partition(self.ptr, pa.ptr) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }

    /// Return true if the table is not in disk order
    pub fn is_wrong_order(&self) -> bool {
        unsafe { libfdisk_sys::fdisk_table_wrong_order(self.ptr) == 1 }
    }

    pub fn iter(&mut self) -> Iter {
        Iter::new(self)
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        unsafe { libfdisk_sys::fdisk_unref_table(self.ptr) }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a mut Table {
    type Item = Partition;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Context {
    pub fn apply_table(&self, table: &Table) -> Result<()> {
        match unsafe { libfdisk_sys::fdisk_apply_table(self.ptr, table.ptr) } {
            0 => Ok(()),
            v => Err(nix::Error::from_errno(nix::errno::from_i32(-v)).into()),
        }
    }
}
