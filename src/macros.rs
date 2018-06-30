macro_rules! compile_prim(
    ($ty:ty, $type_name:ident, $make_constant:ident) => (
impl<'a> Compile<'a> for $ty {
    #[inline(always)]
    fn compile(self, func:&UncompiledFunction<'a>) -> &'a Val {
        use types::consts;
        unsafe {
            from_ptr($make_constant(func.into(), consts::$type_name.into(), self) )
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'a> {
        use types::consts;
        consts::$type_name.clone()
    }
});
    ($ty:ty, $type_name:ident, $make_constant:ident, $cast:ty) => (
#[allow(trivial_numeric_casts)]
impl<'a> Compile<'a> for $ty {
    #[inline(always)]
    fn compile(self, func:&UncompiledFunction<'a>) -> &'a Val {
        use types::consts;
        unsafe {
            from_ptr($make_constant(func.into(), consts::$type_name().into(), self as $cast) )
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'a> {
        use types::consts;
        consts::$type_name().into()
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
        impl<'a, $($arg:Compile<'a>,)* R:Compile<'a>> Compile<'a> for $sig {
            #[inline(always)]
            fn compile(self, func:&UncompiledFunction<'a>) -> &'a Val {
                compile_ptr!(func, self)
            }
            #[inline(always)]
            fn get_type() -> CowType<'a> {
                Type::new_signature(CDecl, &get::<R>(), &mut [$(&get::<$arg>()),*]).into()
            }
        }
        impl<'a, $($arg:Compile<'a>,)* R:Compile<'a>> Compile<'a> for $ext_sig {
            #[inline(always)]
            fn compile(self, func:&UncompiledFunction<'a>) -> &'a Val {
                compile_ptr!(func, self)
            }
            #[inline(always)]
            fn get_type() -> CowType<'a> {
                Type::new_signature(CDecl, &get::<R>(), &mut [$(&get::<$arg>()),*]).into()
            }
        }
    )
);
macro_rules! compile_tuple(
    ($($ty:ident),+ => $($name:ident),+) => (
        impl<'a, $($ty),+> Compile<'a> for ($($ty),+) where $($ty:Compile<'a>),+ {
            #[inline(always)]
            fn compile(self, func:&UncompiledFunction<'a>) -> &'a Val {
                let ($($name),+) = self;
                let ty = get::<($($ty),+)>();
                let tuple = Val::new(func, &ty);
                let ($($name),+) = ($(func.insn_of($name)),+);
                let mut fields = ty.fields();
                $(func.insn_store_relative(tuple, fields.next().unwrap().get_offset(), $name);)+
                tuple
            }
            #[inline(always)]
            fn get_type() -> CowType<'a> {
                use std::mem;
                let mut types = [$(&*get::<$ty>()),+];
                let ty = Type::new_struct(&mut types);
                unsafe {
                    jit_type_set_size_and_alignment((&ty).into(), mem::size_of::<Self>() as i64, mem::align_of::<Self>() as i64);
                }
                ty.into()
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
    (&$name:ident = $alias:ty) => (
        use std::mem::transmute as cast;
        impl Eq for $name {}
        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                unsafe { cast::<_, isize>(self) == cast(other) }
            }
        }
        impl<'a> From<&'a $name> for $alias {
            fn from(ty: &'a $name) -> $alias {
                unsafe { cast(ty) }
            }
        }
        impl<'a> From<&'a mut $name> for $alias {
            fn from(ty: &'a mut $name) -> $alias {
                unsafe { cast(ty) }
            }
        }
        impl<'a> From<$alias> for &'a $name {
            fn from(ty: $alias) -> &'a $name {
                unsafe { cast(ty) }
            }
        }
    );
    ($name:ident, $field:ident: $pointer_ty:ty) => (
        impl<'a> From<&'a mut $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<&'a $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$pointer_ty> for $name {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name {
                $name {
                    $field: ptr
                }
            }
        }
    );
    ($name:ident, $field:ident: $pointer_ty:ty, $($ofield:ident = $expr:expr),*) => (
        impl<'a> From<&'a mut $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<&'a $name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$name> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name) -> $pointer_ty {
                thing.$field
            }
        }
        impl From<$pointer_ty> for $name {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name {
                $name {
                    $field: ptr,
                    $($ofield: $expr),*
                }
            }
        }
    );
    ($name:ident<$ty:ident>, $field:ident: $pointer_ty:ty, $($ofield:ident = $expr:expr),*) => (
        impl<'a, $ty> From<&'a mut $name<$ty>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name<$ty>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a, $ty> From<&'a $name<$ty>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name<$ty>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<$ty> From<$name<$ty>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name<$ty>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<$ty> From<$pointer_ty> for $name<$ty> {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name<$ty> {
                $name {
                    $field: ptr,
                    $($ofield: $expr),*
                }
            }
        }
    );
    (contra $name:ident, $field:ident: $pointer_ty:ty) => (
        impl<'a, 'b> From<&'a mut $name<'b>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a mut $name<'b>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a, 'b> From<&'a $name<'b>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: &'a $name<'b>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<$name<'a>> for $pointer_ty {
            /// Convert into a native pointer
            fn from(thing: $name<'a>) -> $pointer_ty {
                thing.$field
            }
        }
        impl<'a> From<$pointer_ty> for $name<'a> {
            /// Convert from a native pointer
            fn from(ptr: $pointer_ty) -> $name<'a> {
                $name {
                    $field: ptr,
                    marker: PhantomData
                }
            }
        }
    )
);
macro_rules! builtin_type(
    ($c_name:ident -> $rust_name:ident) => (
        pub fn $rust_name() -> StaticType {
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
