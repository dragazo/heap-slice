use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};
use core::hash::{Hash, Hasher};
use core::cmp::Ordering;
use core::{str, fmt};

#[derive(Default, Clone)]
pub struct HeapStr(crate::HeapSlice<u8>);

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
        std::thread::spawn(move || {
            assert_eq!(v, content);
            assert_eq!(v.deref(), content);
            assert_eq!(v.deref_mut(), content);
            assert_eq!(v.as_ref(), content);
            assert_eq!(v.as_mut(), content);
            assert_eq!(<HeapStr as Borrow<str>>::borrow(&v), content);
            assert_eq!(<HeapStr as BorrowMut<str>>::borrow_mut(&mut v), content);
        }).join().unwrap();
    }
    assert_eq!(HeapStr::default(), "");
}
