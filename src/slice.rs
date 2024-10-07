use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::cmp::Ordering;
use core::{slice, fmt};
use core::ptr::NonNull;

pub struct HeapSlice<T>(NonNull<u8>, PhantomData<T>);

unsafe impl<T: Send> Send for HeapSlice<T> {}
unsafe impl<T: Sync> Sync for HeapSlice<T> {}

impl<T: Clone> From<&[T]> for HeapSlice<T> {
    fn from(value: &[T]) -> Self {
        if value.is_empty() {
            return Self::default();
        }

        let align = align_of::<usize>().max(align_of::<T>());
        let size = align + size_of_val(value);
        unsafe {
            let ptr = alloc::alloc::alloc(alloc::alloc::Layout::from_size_align(size, align).unwrap_unchecked());
            (ptr as *mut usize).write(value.len());
            let mut p = ptr.add(align) as *mut T;
            for v in value {
                p.write(v.clone());
                p = p.add(1);
            }
            Self(NonNull::new_unchecked(ptr), PhantomData)
        }
    }
}

impl<T> Default for HeapSlice<T> {
    fn default() -> Self {
        Self(unsafe { NonNull::new_unchecked(sptr::invalid_mut(1)) }, PhantomData)
    }
}

impl<T> Deref for HeapSlice<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        if self.0.as_ptr() as usize != 1 {
            let align = align_of::<usize>().max(align_of::<T>());
            unsafe {
                slice::from_raw_parts(self.0.as_ptr().add(align) as *const T, (self.0.as_ptr() as *const usize).read())
            }
        } else {
            &[]
        }
    }
}

impl<T> DerefMut for HeapSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                let align = align_of::<usize>().max(align_of::<T>());
                slice::from_raw_parts_mut(self.0.as_ptr().add(align) as *mut T, (self.0.as_ptr() as *const usize).read())
            }
        } else {
            &mut []
        }
    }
}

impl<T> Drop for HeapSlice<T> {
    fn drop(&mut self) {
        if self.0.as_ptr() as usize != 1 {
            unsafe {
                let values = &mut **self;

                let align = align_of::<usize>().max(align_of::<T>());
                let size = align + size_of_val(values);

                for value in values {
                    (value as *mut T).drop_in_place();
                }

                alloc::alloc::dealloc(self.0.as_ptr(), alloc::alloc::Layout::from_size_align(size, align).unwrap_unchecked());
            }
        }
    }
}

impl<T> AsRef<[T]> for HeapSlice<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for HeapSlice<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> Borrow<[T]> for HeapSlice<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> BorrowMut<[T]> for HeapSlice<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Hash> Hash for HeapSlice<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T: fmt::Debug> fmt::Debug for HeapSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &**self)
    }
}

impl<T: Clone> Clone for HeapSlice<T> {
    fn clone(&self) -> Self {
        Self::from(&**self)
    }
}

impl<T: PartialEq, U: AsRef<[T]>> PartialEq<U> for HeapSlice<T> {
    fn eq(&self, other: &U) -> bool {
        (**self).eq(other.as_ref())
    }
}

impl<T: Eq> Eq for HeapSlice<T> {}

impl<T: PartialOrd, U: AsRef<[T]>> PartialOrd<U> for HeapSlice<T> {
    fn partial_cmp(&self, other: &U) -> Option<Ordering> {
        (**self).partial_cmp(other.as_ref())
    }
}

impl<T: Ord> Ord for HeapSlice<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).cmp(&**other)
    }
}

#[test]
fn test_basic() {
    extern crate std;
    use alloc::string::String;

    assert_eq!(size_of::<HeapSlice<u8>>(), size_of::<usize>());
    assert_eq!(size_of::<Option<HeapSlice<u8>>>(), size_of::<usize>());

    macro_rules! assert_implements {
        ($t:ty : $($tt:tt)*) => {{
            fn f<T: $($tt)*>() {}
            f::<$t>();
        }};
    }

    assert_implements!(HeapSlice<u8>: Send + Sync + fmt::Debug + Clone + PartialEq + Eq + PartialOrd + Ord + Deref<Target = [u8]> + DerefMut + AsRef<[u8]> + AsMut<[u8]> + Borrow<[u8]> + BorrowMut<[u8]> + Hash + Default);

    for content in [b"".as_slice(), b"h", b"he", b"hel", b"help", b"help me obi-wan kenobi, you're my only hope"] {
        let mut v = HeapSlice::from(content);
        assert_eq!(v, content);
        assert_eq!(v.deref(), content);
        assert_eq!(v.deref_mut(), content);
        assert_eq!(v.as_ref(), content);
        assert_eq!(v.as_mut(), content);
        assert_eq!(<HeapSlice<u8> as Borrow<[u8]>>::borrow(&v), content);
        assert_eq!(<HeapSlice<u8> as BorrowMut<[u8]>>::borrow_mut(&mut v), content);
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
            assert_eq!(<HeapSlice<u8> as Borrow<[u8]>>::borrow(&v), content);
            assert_eq!(<HeapSlice<u8> as BorrowMut<[u8]>>::borrow_mut(&mut v), content);
            if v.is_empty() {
                assert_eq!(v.0.as_ptr() as usize, 1);
            } else {
                assert_ne!(v.0.as_ptr() as usize, 1);
                assert_eq!((v.0.as_ptr() as usize) % align_of::<usize>(), 0);
            }
        }).join().unwrap();
    }

    let vv = HeapSlice::<u8>::default();
    assert_eq!(vv, &[]);
    assert_eq!(vv.0.as_ptr() as usize, 1);

    let x = HeapSlice::<String>::from([String::from("hello"), String::from("world")].as_slice());
    assert_eq!(x.len(), 2);
    assert_eq!(x[0], "hello");
    assert_eq!(x[1], "world");

    let xx = HeapSlice::<String>::default();
    assert_eq!(xx, &[]);
}
