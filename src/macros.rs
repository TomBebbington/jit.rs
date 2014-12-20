macro_rules! compile_prim(
    ($ty:ty, $type_name:ident, $make_constant:ident) => (
impl Compile for $ty {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
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
    ($ty:ty, $type_name:ident, $make_constant:ident, $cast:ty) => (
impl Compile for $ty {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
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
);

macro_rules! compile_func(
    (fn($($arg:ident),*) -> $ret:ty, $sig:ty, $ext_sig:ty) => (
        impl<$($arg:Compile,)* R:Compile> Compile for $sig {
            #[inline(always)]
            fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
                let ptr = (self as *const $sig).to_uint().compile(func);
                func.insn_convert(&ptr, get::<$sig>(), false)
            }
            #[inline(always)]
            fn jit_type(_:Option<$sig>) -> Type {
                Type::create_signature(CDECL, get::<R>(), [$(get::<$arg>()),*][mut])
            }
        }
        impl<$($arg:Compile,)* R:Compile> Compile for $ext_sig {
            #[inline(always)]
            fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
                let ptr = (self as *const $ext_sig).to_uint().compile(func);
                func.insn_convert(&ptr, get::<$ext_sig>(), false)
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
                let ($(ref $name),+) = ($(func.insn_of($name)),+);
                let mut fields = ty.fields();
                $(func.insn_store_relative(&tuple, fields.next().unwrap().get_offset() as int, $name);)+
                tuple
            }
            #[inline(always)]
            fn jit_type(_:Option<($($ty),+)>) -> Type {
                Type::create_struct([$(get::<$ty>()),+][mut])
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
#[macro_export]
macro_rules! jit(
    (struct {
        $($name:expr: $ty:ty),*
    }) => ({
        let structure = Type::create_struct([
            $(get::<$ty>()),*
        ].as_mut_slice());
        structure.set_names(&[$($name),*]);
        structure
    });
    (union {
        $($name:expr: $ty:ty),+
    }) => ({
        let structure = Type::create_union([
            $(get::<$ty>()),+
        ].as_mut_slice());
        structure.set_names(&[$($name),+]);
        structure
    });
    ($func:ident, return) => (
        $func.insn_default_return()
    );
    ($func:ident, return $($t:tt)+) => (
        $func.insn_return(jit!($($t)+))
    );
    ($func:ident, $var:ident += $num:expr) => (
        $func.insn_store($var, &$func.insn_add($var, $num));
    );
    ($func:ident, $var:ident -= $num:expr) => (
        $func.insn_store($var, &$func.insn_sub($var, $num));
    );
    ($func:ident, $var:ident *= $num:expr) => (
        $func.insn_store($var, &$func.insn_mul($var, $num));
    );
    ($func:ident, $var:ident /= $num:expr) => (
        $func.insn_store($var, &$func.insn_div($var, $num));
    );
    ($func:ident, $a:expr + $b:expr) => (
        $func.insn_add($a, $b)
    );
    ($func:ident, $a:expr - $b:expr) => (
        $func.insn_sub($a, $b)
    );
    ($func:ident, $a:expr * $b:expr) => (
        $func.insn_mul($a, $b)
    );
    ($func:ident, $a:expr / $b:expr) => (
        $func.insn_div($a, $b)
    );
    ($func:ident, $a:expr % $b:expr) => (
        $func.insn_rem($a, $b)
    );
    ($func:ident, $var:ident = $val:expr) => (
        $func.insn_store($var, $val);
    );
    ($func:ident, *$var:ident) => (
        $func.insn_load($var)
    );
    ($func:ident, call($call:expr,
        $($arg:expr),+
    )) => (
        $func.insn_call(None::<String>, $call, None, [$($arg),+].as_mut_slice())
    );
    ($func:ident, jump_table($value:expr,
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
);
#[macro_export]
macro_rules! jit_func(
    ($ctx:expr, $func:ident, fn $name:ident($($arg:ident:$ty:ty),*) -> $ret:ty $value:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [$(get::<$ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0u;
            $(let ref $arg = $func[{i += 1; i - 1}];)+
            $value
        })
    });
    ($ctx:expr, $func:ident, fn $name:ident($($arg:ident:$arg_ty:ty),*) -> $ret:ty $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [$(get::<$arg_ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0u;
            $(let ref $arg = $func[{i += 1; i - 1}];)*
            $value
        }).with::<($($arg_ty),*), $ret, _>(|$comp_func|
            $comp)
    });
);