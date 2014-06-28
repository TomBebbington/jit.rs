use bindings::{
    jit_type_void_ptr,
    jit_type_ubyte,
    jit_type_sbyte,
    jit_type_ushort,
    jit_type_short,
    jit_type_uint,
    jit_type_int,
    jit_type_nuint,
    jit_type_nint,
    jit_type_float32,
    jit_type_float64,
    jit_type_long,
    jit_type_ulong,
    jit_type_sys_bool,
    jit_type_sys_char,

    jit_value_create_long_constant,
    jit_value_create_nint_constant,
    jit_value_create_float64_constant,
    jit_value_create_float32_constant
};
use function::{
    CDECL,
    Function
};
use libc::c_long;
use value::Value;
use types::Type;
use util::NativeRef;
/// A type that can be compiled into a LibJIT representation
pub trait Compile {
    /// Get a JIT representation of this value
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a>;
    /// Get the JIT type repr of the value
    fn jit_type(_:Option<Self>) -> Type;
}
impl Compile for () {
    fn compile(&self, func:&Function) -> Value {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_void_ptr, 0))
        }
    }
    #[inline]
    fn jit_type(_:Option<()>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_void_ptr)
        }
    }
}
compile_prim!(f64, jit_type_float64, jit_value_create_float64_constant)
compile_prim!(f32, jit_type_float32, jit_value_create_float32_constant)
compile_prim!(int, jit_type_nint, jit_value_create_nint_constant, c_long)
compile_prim!(uint,jit_type_nuint, jit_value_create_nint_constant, c_long)
compile_prim!(i64, jit_type_long, jit_value_create_long_constant, c_long)
compile_prim!(u64, jit_type_ulong, jit_value_create_long_constant, c_long)
compile_prim!(i32, jit_type_int, jit_value_create_nint_constant, c_long)
compile_prim!(u32, jit_type_uint, jit_value_create_nint_constant, c_long)
compile_prim!(i16, jit_type_short, jit_value_create_nint_constant, c_long)
compile_prim!(u16, jit_type_ushort, jit_value_create_nint_constant, c_long)
compile_prim!(i8, jit_type_sbyte, jit_value_create_nint_constant, c_long)
compile_prim!(u8, jit_type_ubyte, jit_value_create_nint_constant, c_long)
compile_prim!(bool, jit_type_sys_bool, jit_value_create_nint_constant, c_long)
compile_prim!(char, jit_type_sys_char, jit_value_create_nint_constant, c_long)
impl<'t> Compile for &'t str {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let cstring_t = ::get_type::<&'t str>();
        let strlen_i = self.len().compile(func);
        let bufptr = Value::new(func, cstring_t);
        func.insn_store(&bufptr, &func.insn_alloca(&strlen_i));
        for i in range(0, self.len()) {
            let char_i = self.char_at(i).compile(func);
            func.insn_store_relative(&bufptr, i as int, &char_i);
        }
        let null_term = '\0'.compile(func);
        func.insn_store_relative(&bufptr, self.len() as int, &null_term);
        bufptr
    }
    #[inline]
    fn jit_type(_:Option<&'t str>) -> Type {
        Type::create_pointer(::get_type::<char>())
    }
}
impl Compile for String {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        self.as_slice().compile(func)
    }
    #[inline]
    fn jit_type(_:Option<String>) -> Type {
        unsafe {
            Type::create_pointer(NativeRef::from_ptr(jit_type_sys_char))
        }
    }
}
impl<T:Compile> Compile for *T {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&self.to_uint().compile(func), ::get_type::<T>(), false)
    }
    #[inline]
    fn jit_type(_:Option<*T>) -> Type {
        Type::create_pointer(::get_type::<T>())
    }
}
impl<R:Compile> Compile for fn() -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn() -> R).to_uint().compile(func), ::get_type::<fn() -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn() -> R>) -> Type {
        Type::create_signature(CDECL, ::get_type::<R>(), &mut [])
    }
}
impl<A:Compile, R:Compile> Compile for fn(A) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn(A) -> R).to_uint().compile(func), ::get_type::<fn(A) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A) -> R>) -> Type {
        Type::create_signature(CDECL, ::get_type::<R>(), &mut [::get_type::<A>()])
    }
}
impl<A:Compile, B:Compile, R:Compile> Compile for fn(A, B) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn(A, B) -> R).to_uint().compile(func), ::get_type::<fn(A, B) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A, B) -> R>) -> Type {
        Type::create_signature(CDECL, ::get_type::<R>(), &mut [::get_type::<A>(), ::get_type::<B>()])
    }
}
impl<A:Compile, B:Compile, C:Compile, R:Compile> Compile for fn(A, B, C) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn(A, B, C) -> R).to_uint().compile(func), ::get_type::<fn(A, B, C) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A, B, C) -> R>) -> Type {
        Type::create_signature(CDECL, ::get_type::<R>(), &mut [::get_type::<A>(), ::get_type::<B>(), ::get_type::<C>()])
    }
}