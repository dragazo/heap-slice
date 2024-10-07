`Box<T>` is always the same size as a pointer for normal, sized `T` values.
But for unsized values like `Box<str>`, `Box<[u8]>`, or in general `Box<[T]>`, it's actually 2 pointers!
This is due to a nuance of Rust where fat pointers (as used in slices) store their extra data (e.g., length) in the stack.
