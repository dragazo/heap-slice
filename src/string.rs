use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::hash::{Hash, Hasher};
use core::cmp::Ordering;
use core::{str, fmt};

/// Basically [`Box<str>`](alloc::boxed::Box), but smaller.
#[derive(Default, Clone)]
pub struct HeapStr(crate::HeapSlice<u8>);

impl HeapStr {
    /// Attempts to construct a new [`HeapStr`] from its underlying [`HeapSlice<u8>`](crate::HeapSlice) container.
    /// No allocations are performed.
    pub fn from_utf8(content: crate::HeapSlice<u8>) -> Result<Self, core::str::Utf8Error> {
        Ok(str::from_utf8(&content)?.into())
    }
    /// As [`HeapStr::from_utf8`], but does not perform its UTF-8 conformance check.
    pub unsafe fn from_utf8_unchecked(content: crate::HeapSlice<u8>) -> Self {
        str::from_utf8_unchecked(&content).into()
    }
    /// Extracts the underlying [`HeapSlice<u8>`](crate::HeapSlice) container.
    pub fn into_bytes(self) -> crate::HeapSlice<u8> {
        self.0
    }
}

impl From<&str> for HeapStr {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().into())
    }
}

impl Deref for HeapStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe {
            str::from_utf8_unchecked(&self.0)
        }
    }
}

impl DerefMut for HeapStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            str::from_utf8_unchecked_mut(&mut self.0)
        }
    }
}

impl AsRef<str> for HeapStr {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for HeapStr {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl Borrow<str> for HeapStr {
    fn borrow(&self) -> &str {
        self
    }
}

impl BorrowMut<str> for HeapStr {
    fn borrow_mut(&mut self) -> &mut str {
        self
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

        let mut vv = v.into_bytes();
        assert_eq!(vv, content.as_bytes());
        assert_eq!(vv.deref(), content.as_bytes());
        assert_eq!(vv.deref_mut(), content.as_bytes());
        assert_eq!(vv.as_ref(), content.as_bytes());
        assert_eq!(vv.as_mut(), content.as_bytes());
        assert_eq!(<crate::HeapSlice<u8> as Borrow<[u8]>>::borrow(&vv), content.as_bytes());
        assert_eq!(<crate::HeapSlice<u8> as BorrowMut<[u8]>>::borrow_mut(&mut vv), content.as_bytes());

        let mut vvv = HeapStr::from_utf8(vv).unwrap();
        assert_eq!(vvv, content);
        assert_eq!(vvv.deref(), content);
        assert_eq!(vvv.deref_mut(), content);
        assert_eq!(vvv.as_ref(), content);
        assert_eq!(vvv.as_mut(), content);
        assert_eq!(<HeapStr as Borrow<str>>::borrow(&vvv), content);
        assert_eq!(<HeapStr as BorrowMut<str>>::borrow_mut(&mut vvv), content);

        std::thread::spawn(move || {
            assert_eq!(vvv, content);
            assert_eq!(vvv.deref(), content);
            assert_eq!(vvv.deref_mut(), content);
            assert_eq!(vvv.as_ref(), content);
            assert_eq!(vvv.as_mut(), content);
            assert_eq!(<HeapStr as Borrow<str>>::borrow(&vvv), content);
            assert_eq!(<HeapStr as BorrowMut<str>>::borrow_mut(&mut vvv), content);
        }).join().unwrap();
    }
    assert_eq!(HeapStr::default(), "");
}
