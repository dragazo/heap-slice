use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::hash::{Hash, Hasher};
use core::cmp::Ordering;
use core::{slice, fmt};
use core::ptr::NonNull;

pub struct HeapBytes(NonNull<u8>);

unsafe impl Send for HeapBytes {}
unsafe impl Sync for HeapBytes {}

impl From<&[u8]> for HeapBytes {
    fn from(value: &[u8]) -> Self {
        Self(if !value.is_empty() {
            unsafe {
                let ptr = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(size_of::<usize>() + value.len(), align_of::<usize>()).unwrap_unchecked());
                (ptr as *mut usize).write(value.len());
                ptr.add(size_of::<usize>()).copy_from_nonoverlapping(value.as_ptr(), value.len());
                NonNull::new_unchecked(ptr)
            }
        } else {
            unsafe { NonNull::new_unchecked(sptr::invalid_mut(1)) }
        })
    }
}

impl Deref for HeapBytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                slice::from_raw_parts(self.0.as_ptr().add(size_of::<usize>()), (self.0.as_ptr() as *const usize).read())
            }
        } else {
            &[]
        }
    }
}

impl DerefMut for HeapBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                slice::from_raw_parts_mut(self.0.as_ptr().add(size_of::<usize>()), (self.0.as_ptr() as *const usize).read())
            }
        } else {
            &mut []
        }
    }
}

impl Drop for HeapBytes {
    fn drop(&mut self) {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                alloc::alloc::dealloc(self.0.as_ptr(), alloc::alloc::Layout::from_size_align(size_of::<usize>() + (**self).len(), align_of::<usize>()).unwrap_unchecked());
            }
        }
    }
}

impl Default for HeapBytes {
    fn default() -> Self {
        Self::from([].as_slice())
    }
}

impl AsRef<[u8]> for HeapBytes {
    fn as_ref(&self) -> &[u8] {
        &**self
    }
}

impl AsMut<[u8]> for HeapBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut **self
    }
}

impl Borrow<[u8]> for HeapBytes {
    fn borrow(&self) -> &[u8] {
        &**self
    }
}

impl BorrowMut<[u8]> for HeapBytes {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut **self
    }
}

impl Hash for HeapBytes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl fmt::Debug for HeapBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

impl Clone for HeapBytes {
    fn clone(&self) -> Self {
        Self::from(&**self)
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for HeapBytes {
    fn eq(&self, other: &T) -> bool {
        (**self).eq(other.as_ref())
    }
}

impl Eq for HeapBytes {}

impl<T: AsRef<[u8]>> PartialOrd<T> for HeapBytes {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        (**self).partial_cmp(other.as_ref())
    }
}

impl Ord for HeapBytes {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

#[test]
fn test_basic() {
    extern crate std;

    assert_eq!(size_of::<HeapBytes>(), size_of::<usize>());
    assert_eq!(size_of::<Option<HeapBytes>>(), size_of::<usize>());

    fn assert_traits<T: Send + Sync + fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Deref<Target = [u8]> + DerefMut + AsRef<[u8]> + AsMut<[u8]> + Borrow<[u8]> + BorrowMut<[u8]> + Hash + Default>() {}
    assert_traits::<HeapBytes>();

    for content in [b"".as_slice(), b"h", b"he", b"hel", b"help", b"help me obi-wan kenobi, you're my only hope"] {
        let mut v = HeapBytes::from(content);
        assert_eq!(v, content);
        assert_eq!(v.deref(), content);
        assert_eq!(v.deref_mut(), content);
        assert_eq!(v.as_ref(), content);
        assert_eq!(v.as_mut(), content);
        assert_eq!(<HeapBytes as Borrow<[u8]>>::borrow(&v), content);
        assert_eq!(<HeapBytes as BorrowMut<[u8]>>::borrow_mut(&mut v), content);
        if v.is_empty() {
            assert_eq!(v.0.as_ptr() as usize, 1);
        } else {
            assert_ne!(v.0.as_ptr() as usize, 1);
            assert_eq!((v.0.as_ptr() as usize) % align_of::<usize>(), 0);
        }
        std::thread::spawn(move || {
            assert_eq!(v, content);
            assert_eq!(v.deref(), content);
            assert_eq!(v.deref_mut(), content);
            assert_eq!(v.as_ref(), content);
            assert_eq!(v.as_mut(), content);
            assert_eq!(<HeapBytes as Borrow<[u8]>>::borrow(&v), content);
            assert_eq!(<HeapBytes as BorrowMut<[u8]>>::borrow_mut(&mut v), content);
            if v.is_empty() {
                assert_eq!(v.0.as_ptr() as usize, 1);
            } else {
                assert_ne!(v.0.as_ptr() as usize, 1);
                assert_eq!((v.0.as_ptr() as usize) % align_of::<usize>(), 0);
            }
        }).join().unwrap();
    }

    let vv = HeapBytes::default();
    assert_eq!(vv, &[]);
    assert_eq!(vv.0.as_ptr() as usize, 1);
}
