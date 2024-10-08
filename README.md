As we know, [`Box<T>`](alloc::boxed::Box) is a simple wrapper for a pointer into the heap.
So it's always 8-bytes on the stack, right?
Wrong!
In order to make many unsized types work (e.g., `[T]` and `str`), Rust stores some extra information (e.g., length) in the pointer itself.
This creates a larger stack structure called a "fat" pointer.
Thus, `Box<[T]>` and `Box<str>` actually take up 16 bytes on the stack!

For some applications, this can be very undesirable (e.g., one large enum variant will make the whole enum large).
This crate provides two new types called [`HeapSlice<T>`] and [`HeapStr`] which solve this problem by storing the length in the heap rather than the stack.
Thus, these types serve as a drop-in replacement for `Box<[T]>` and `Box<str>`, but only take 8 bytes on the stack rather than 16.

Both types support [`Option`] size optimization and a special non-allocating case for empty slices/strings.

## `no_std`

This crate supports building in `no_std` environments out of the box, but naturally `alloc` is still required.
