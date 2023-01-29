/// Returns the offset of a field in a struct, works like `offsetof` macro
/// in C.
///
/// Structs with `#[repr(Rust)]` are not guaranteed to have a stable layout,
/// so [`offset_of!`] might not work as expected. `#[repr(C)]` is recommended
/// for structs that should be used with [`offset_of!`].
#[macro_export]
macro_rules! offset_of {
    ($container:ty, $field:ident) => {{
        let base = core::mem::MaybeUninit::<$container>::uninit().as_ptr();
        let field = core::ptr::addr_of!((*base).$field);
        field as isize - base as isize
    }};
}
