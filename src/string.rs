use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::hash::{Hash, Hasher};
use core::{slice, str, fmt};
use core::cmp::Ordering;
use core::ptr::NonNull;

pub struct HeapStr(NonNull<u8>);

unsafe impl Send for HeapStr {}
unsafe impl Sync for HeapStr {}

impl From<&str> for HeapStr {
    fn from(value: &str) -> Self {
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

impl Deref for HeapStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                str::from_utf8_unchecked(slice::from_raw_parts(self.0.as_ptr().add(size_of::<usize>()), (self.0.as_ptr() as *const usize).read()))
            }
        } else {
            ""
        }
    }
}

impl DerefMut for HeapStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(self.0.as_ptr().add(size_of::<usize>()), (self.0.as_ptr() as *const usize).read()))
            }
        } else {
            unsafe {
                str::from_utf8_unchecked_mut(&mut [])
            }
        }
    }
}

impl Drop for HeapStr {
    fn drop(&mut self) {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                alloc::alloc::dealloc(self.0.as_ptr(), alloc::alloc::Layout::from_size_align(size_of::<usize>() + (**self).len(), align_of::<usize>()).unwrap_unchecked());
            }
        }
    }
}

impl Default for HeapStr {
    fn default() -> Self {
        Self::from("")
    }
}

impl AsRef<str> for HeapStr {
    fn as_ref(&self) -> &str {
        &**self
    }
}

impl AsMut<str> for HeapStr {
    fn as_mut(&mut self) -> &mut str {
        &mut **self
    }
}

impl Borrow<str> for HeapStr {
    fn borrow(&self) -> &str {
        &**self
    }
}

impl BorrowMut<str> for HeapStr {
    fn borrow_mut(&mut self) -> &mut str {
        &mut **self
    }
}

impl Hash for HeapStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl fmt::Debug for HeapStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

impl fmt::Display for HeapStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &**self)
    }
}

impl Clone for HeapStr {
    fn clone(&self) -> Self {
        Self::from(&**self)
    }
}

impl<T: AsRef<str>> PartialEq<T> for HeapStr {
    fn eq(&self, other: &T) -> bool {
        (**self).eq(other.as_ref())
    }
}

impl Eq for HeapStr {}

impl<T: AsRef<str>> PartialOrd<T> for HeapStr {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        (**self).partial_cmp(other.as_ref())
    }
}

impl Ord for HeapStr {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

#[test]
fn test_basic() {
    extern crate std;

    assert_eq!(size_of::<HeapStr>(), size_of::<usize>());
    assert_eq!(size_of::<Option<HeapStr>>(), size_of::<usize>());

    fn assert_traits<T: Send + Sync + fmt::Debug + fmt::Display + Clone + PartialEq + Eq + PartialOrd + Ord + Deref<Target = str> + DerefMut + AsRef<str> + AsMut<str> + Borrow<str> + BorrowMut<str> + Hash + Default>() {}
    assert_traits::<HeapStr>();

    for content in ["", "h", "he", "hel", "help", "help me obi-wan kenobi, you're my only hope"] {
        let mut v = HeapStr::from(content);
        assert_eq!(v, content);
        assert_eq!(v.deref(), content);
        assert_eq!(v.deref_mut(), content);
        assert_eq!(v.as_ref(), content);
        assert_eq!(v.as_mut(), content);
        assert_eq!(<HeapStr as Borrow<str>>::borrow(&v), content);
        assert_eq!(<HeapStr as BorrowMut<str>>::borrow_mut(&mut v), content);
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
            assert_eq!(<HeapStr as Borrow<str>>::borrow(&v), content);
            assert_eq!(<HeapStr as BorrowMut<str>>::borrow_mut(&mut v), content);
            if v.is_empty() {
                assert_eq!(v.0.as_ptr() as usize, 1);
            } else {
                assert_ne!(v.0.as_ptr() as usize, 1);
                assert_eq!((v.0.as_ptr() as usize) % align_of::<usize>(), 0);
            }
        }).join().unwrap();
    }
    assert_eq!(HeapStr::default(), "");
}
