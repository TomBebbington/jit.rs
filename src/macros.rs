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
    fn jit_type(_:Option<$ty>) -> Type {
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
            from_ptr($make_constant(func.as_ptr(), consts::$type_name.as_ptr(), *self as $cast) )
        }
    }
    #[inline(always)]
    fn jit_type(_:Option<$ty>) -> Type {
        use types::consts;
        consts::$type_name.clone()
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
            fn jit_type(_:Option<$sig>) -> Type {
                Type::create_signature(CDecl, get::<R>(), [$(get::<$arg>()),*].as_mut_slice())
            }
        }
        impl<$($arg:Compile,)* R:Compile> Compile for $ext_sig {
            #[inline(always)]
            fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
                compile_ptr!(func, self)
            }
            #[inline(always)]
            fn jit_type(_:Option<$ext_sig>) -> Type {
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
                let ty = get::<($($ty),+)>();
                let tuple = Value::new(func, ty.clone());
                let ($($name),+) = ($(func.insn_of($name)),+);
                let mut fields = ty.fields();
                $(func.insn_store_relative(tuple, fields.next().unwrap().get_offset() as isize, $name);)+
                tuple
            }
            #[inline(always)]
            fn jit_type(_:Option<($($ty),+)>) -> Type {
                let mut types = [$(get::<$ty>()),+];
                Type::create_struct(types.as_mut_slice())
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
    ($name:ident, $field:ident, $pointer_ty:ty) => (
        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                self.$field == other.$field
            }
        }
        impl Eq for $name {}
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
        impl<'a> PartialEq for $name<'a> {
            fn eq(&self, other: &$name<'a>) -> bool {
                self.$field == other.$field
            }
        }
        impl<'a> Eq for $name<'a> {}
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
                    marker: $lifetime::<'a>
                }
            }
        }
    )
);
macro_rules! builtin_type(
    ($c_name:ident -> $rust_name:ident) => (
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        pub struct $rust_name;
        impl ::std::ops::Deref for $rust_name {
            type Target = Type;
            fn deref(&self) -> &Type {
                use std::mem;
                unsafe { mem::transmute(&$c_name) }
            }
        }
    )
);
macro_rules! builtin_types(
    ($($c_name:ident -> $rust_name:ident);+) => (
        $(builtin_type!($c_name -> $rust_name);)+
    )
);