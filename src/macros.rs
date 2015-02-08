macro_rules! compile_prim(
    ($ty:ty, $type_name:ident, $make_constant:ident) => (
impl Compile for $ty {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        use types::consts;
        unsafe {
            from_ptr($make_constant(func.as_ptr(), consts::$type_name.as_ptr(), *self) )
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        use types::consts;
        consts::$type_name.clone()
    }
});
    ($ty:ty, $type_name:ident, $make_constant:ident, $cast:ty) => (
impl Compile for $ty {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        use types::consts;
        unsafe {
            from_ptr($make_constant(func.as_ptr(), consts::$type_name().as_ptr(), *self as $cast) )
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        use types::consts;
        consts::$type_name().into_cow()
    }
});
);
macro_rules! compile_ptr(
    ($func:expr, $ptr:expr) => (unsafe {
        use std::mem;
        let ptr = mem::transmute::<_, usize>($ptr);
        ptr.compile($func)
    })
);
macro_rules! compile_func(
    (fn($($arg:ident),*) -> $ret:ty, $sig:ty, $ext_sig:ty) => (
        impl<$($arg:Compile,)* R:Compile> Compile for $sig {
            #[inline(always)]
            fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
                compile_ptr!(func, self)
            }
            #[inline(always)]
            fn get_type() -> CowType<'static> {
                Type::new_signature(CDecl, get::<R>().get(), [$(get::<$arg>().get()),*].as_mut_slice()).into_cow()
            }
        }
        impl<$($arg:Compile,)* R:Compile> Compile for $ext_sig {
            #[inline(always)]
            fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
                compile_ptr!(func, self)
            }
            #[inline(always)]
            fn get_type() -> CowType<'static> {
                get::<$sig>()
            }
        }
    )
);
macro_rules! compile_tuple(
    ($($ty:ident),+ => $($name:ident),+) => (
        impl<$($ty),+> Compile for ($($ty),+) where $($ty:Compile),+ {
            #[inline(always)]
            fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
                let ($(ref $name),+) = *self;
                let ty = get::<($($ty),+)>().get();
                let tuple = Value::new(func, ty);
                let ($($name),+) = ($(func.insn_of($name)),+);
                let mut fields = ty.fields();
                $(func.insn_store_relative(tuple, fields.next().unwrap().get_offset(), $name);)+
                tuple
            }
            #[inline(always)]
            fn get_type() -> CowType<'static> {
                let mut types = [$(get::<$ty>().get()),+];
                Type::new_struct(types.as_mut_slice()).into_cow()
            }
        }
    )
);

macro_rules! compile_prims(
    ($(($ty:ty, $cast: ty) => ($type_name:ident, $make_constant:ident)),+) => (
        $(compile_prim!($ty, $type_name, $make_constant, $cast);)+
    );
);

macro_rules! native_ref(
    ($(#[$attr:meta])* $name:ident { $field:ident: $pointer_ty:ty }) => (
        $(#[$attr])*
        #[derive(PartialEq, Eq)]
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
    ($(#[$attr:meta])* $name:ident $marker:ident { $field:ident: $pointer_ty:ty }) => (
        $(#[$attr])*
        #[derive(PartialEq, Eq)]
        pub struct $name<'a> {
            $field: $pointer_ty,
            marker: $marker<'a>
        }
        impl<'a> NativeRef for $name<'a> {
            #[inline]
            /// Convert to a native pointer
            unsafe fn as_ptr(&self) -> $pointer_ty {
                self.$field
            }
            #[inline]
            /// Convert from a native pointer
            unsafe fn from_ptr(ptr:$pointer_ty) -> $name<'a> {
                $name {
                    $field: ptr,
                    marker: $marker::<'a>
                }
            }
        }
    )
);
macro_rules! builtin_type(
    ($c_name:ident -> $rust_name:ident) => (
        pub fn $rust_name() -> StaticType {
            use util::from_ptr;
            unsafe {
                from_ptr($c_name)
            }
        }
    )
);
macro_rules! builtin_types(
    ($($c_name:ident -> $rust_name:ident);+) => (
        $(builtin_type!($c_name -> $rust_name);)+
    )
);