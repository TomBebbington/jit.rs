#[macro_export]
macro_rules! compile_prim(
    ($ty:ty, $type_name:ident, $make_constant:ident) =>
(impl Compile for $ty {
    #[inline(always)]
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr($make_constant(func.as_ptr(), $type_name, *self) )
        }
    }
    #[inline(always)]
    fn jit_type(_:Option<$ty>) -> Type {
        unsafe {
            NativeRef::from_ptr($type_name)
        }
    }
});
    ($ty:ty, $type_name:ident, $make_constant:ident, $cast:ty) =>
(impl Compile for $ty {
    #[inline(always)]
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr($make_constant(func.as_ptr(), $type_name, *self as $cast) )
        }
    }
    #[inline(always)]
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
            #[inline(always)]
            /// Convert to a native pointer
            unsafe fn as_ptr(&self) -> $pointer_ty {
                self.$field
            }
            #[inline(always)]
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
    (struct {
        $($name:expr: $ty:ty),+
    }) => ({
        let structure = Type::create_struct([
            $(get_type::<$ty>()),+
        ].as_mut_slice());
        structure.set_names(&[$($name),+]);
        structure
    });
    (union {
        $($name:expr: $ty:ty),+
    }) => ({
        let structure = Type::create_union([
            $(get_type::<$ty>()),+
        ].as_mut_slice());
        structure.set_names(&[$($name),+]);
        structure
    });
    ($func:expr, return) => (
        $func.insn_default_return()
    );
    ($func:expr, return $($t:tt)+) => (
        $func.insn_return(jit!($($t)+))
    );
    ($func:expr, call($call:expr,
        $($arg:expr),+
    )) => (
        $func.insn_call(None::<String>, $call, None, [$($arg),+].as_mut_slice())
    );
    ($func:expr, jump_table($value:expr,
        $($label:ident),+
    )) => (
    let ($($label),+) = {
        $(let $label:Label = Label::new($func);)+
        $func.insn_jump_table($value, [
            $($label),+
        ].as_mut_slice());
        ($($label),+)
    });
    ($func:expr, $value:expr) => (
        $func.insn_of(&$value)
    );
    ($ty:ty) => (
        get_type::<$ty>()
    );
)