use core::{
    marker::{
        self,
        PhantomData,
    },
    mem,
};
use std::sync::Arc;

use raw_struct::{
    AccessError,
    AccessMode,
    Copy,
    FromMemoryView,
    MemoryView,
    Reference,
    Viewable,
};

use crate::decrypt::StateDecrypt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EncryptedPtr64<T>
where
    T: 'static + ?Sized,
{
    pub address: u64,
    _dummy: PhantomData<T>,
}

impl<T: ?Sized + 'static> Clone for EncryptedPtr64<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + 'static> marker::Copy for EncryptedPtr64<T> {}

impl<T: ?Sized> EncryptedPtr64<T> {
    #[inline]
    pub fn new(value: u64) -> Self {
        Self {
            address: value,
            _dummy: Default::default(),
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.address == 0
    }

    #[inline]
    pub fn cast<V: ?Sized>(&self) -> EncryptedPtr64<V> {
        EncryptedPtr64::<V> {
            address: self.address,
            _dummy: Default::default(),
        }
    }
}

impl<T: marker::Copy> EncryptedPtr64<T> {
    #[must_use = "copied result must be used"]
    pub fn read_value(
        &self,
        memory: &dyn MemoryView,
        decrypt: &StateDecrypt,
    ) -> Result<Option<T>, AccessError> {
        if self.address > 0 {
            unsafe {
                let decrypted_addr = decrypt.decrypt(self.address);
                let memory = T::read_object(memory, decrypted_addr).map_err(|err| AccessError {
                    source: err,
                    member: None,
                    object: "T".into(),
                    mode: AccessMode::Read,
                    offset: decrypted_addr,
                    size: mem::size_of::<T>(),
                })?;
                Ok(Some(memory))
            }
        } else {
            Ok(None)
        }
    }
}

impl<T: ?Sized + Viewable<T>> EncryptedPtr64<T> {
    #[must_use]
    pub fn value_reference(
        &self,
        memory: Arc<dyn MemoryView>,
        decrypt: &StateDecrypt,
    ) -> Option<Reference<T>> {
        if self.address > 0 {
            unsafe {
                let decrypted_addr = decrypt.decrypt(self.address);
                Some(Reference::new(memory, decrypted_addr))
            }
        } else {
            None
        }
    }

    #[must_use = "copied result must be used"]
    pub fn value_copy(
        &self,
        memory: &dyn MemoryView,
        decrypt: &StateDecrypt,
    ) -> Result<Option<Copy<T>>, AccessError> {
        if self.address > 0 {
            unsafe {
                let decrypted_addr = decrypt.decrypt(self.address);
                let memory =
                    T::Memory::read_object(memory, decrypted_addr).map_err(|err| AccessError {
                        source: err,
                        member: None,
                        object: T::name(),
                        mode: AccessMode::Read,
                        offset: decrypted_addr,
                        size: mem::size_of::<T::Memory>(),
                    })?;
                Ok(Some(Copy::new(memory)))
            }
        } else {
            Ok(None)
        }
    }
}

impl<T: ?Sized> From<u64> for EncryptedPtr64<T> {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
