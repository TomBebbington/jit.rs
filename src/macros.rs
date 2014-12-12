#[macro_export]
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
)

macro_rules! compile_func(
    (fn($($arg:ident),*) -> $ret:ty, $sig:ty) => (
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
    })
)

macro_rules! compile_prims(
    ($(($ty:ty, $cast: ty) => ($type_name:ident, $make_constant:ident)),+) => (
        $(compile_prim!($ty, $type_name, $make_constant, $cast))+
    );
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
            unsafe fn from_ptr<'a>(ptr:$pointer_ty) -> $name<'a> {
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
)