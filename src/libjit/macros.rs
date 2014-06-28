#[macro_export]
macro_rules! compile_prim(
    ($ty:ty, $type_name:ident, $make_constant:ident) =>
(impl Compile for $ty {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr($make_constant(func.as_ptr(), $type_name, *self) )
        }
    }
    #[inline]
    fn jit_type(_:Option<$ty>) -> Type {
        unsafe {
            NativeRef::from_ptr($type_name)
        }
    }
});
    ($ty:ty, $type_name:ident, $make_constant:ident, $cast:ty) =>
(impl Compile for $ty {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr($make_constant(func.as_ptr(), $type_name, *self as $cast) )
        }
    }
    #[inline]
    fn jit_type(_:Option<$ty>) -> Type {
        unsafe {
            NativeRef::from_ptr($type_name)
        }
    }
});
)
#[macro_export]
macro_rules! native_ref(
    ($name:ident, $field:ident, $pointer_ty:ty) => (
        #[deriving(PartialEq)]
        pub struct $name {
            $field: $pointer_ty
        }
        impl NativeRef for $name {
            #[inline]
            /// Convert to a native pointer
            unsafe fn as_ptr(&self) -> $pointer_ty {
                self.$field
            }
            #[inline]
            /// Convert from a native pointer
            unsafe fn from_ptr(ptr:$pointer_ty) -> $name {
                $name {
                    $field: ptr
                }
            }
        }
    );
    ($name:ident, $field:ident, $pointer_ty:ty, $lifetime:ident) => (
        #[deriving(PartialEq)]
        pub struct $name<'a> {
            $field: $pointer_ty,
            marker: $lifetime<'a>
        }
        impl<'a> NativeRef for $name<'a> {
            #[inline]
            /// Convert to a native pointer
            unsafe fn as_ptr(&self) -> $pointer_ty {
                self.$field
            }
            #[inline]
            /// Convert from a native pointer
            unsafe fn from_ptr(ptr:$pointer_ty) -> $name {
                $name {
                    $field: ptr,
                    marker: $lifetime::<'a>
                }
            }
        }
    )
)
#[macro_export]
macro_rules! jit(
    ($ty:ty) => (
        ::jit::get_type::<$ty>()
    );
    ($value:expr, $func:expr) => (
        $value.compile($func)
    );
)