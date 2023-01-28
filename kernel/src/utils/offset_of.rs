/// Returns the offset of a field in a struct, works like `offsetof` macro
/// in C.
///
/// Structs with `#[repr(Rust)]` are not guaranteed to have a stable layout,
/// so [`offset_of!`] might not work as expected. `#[repr(C)]` is recommended
/// for structs that should be used with [`offset_of!`].
#[macro_export]
macro_rules! offset_of {
    ($container:ty, $field:ident) => {
        unsafe { &(*(0 as *const $container)).$field as *const _ as usize }
    };
}
