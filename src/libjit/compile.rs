use bindings::{
    jit_nint,

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
use value::Value;
use types::{
    Type,
    TypeKind,
    Void,
    Float32,
    Float64,
    NInt,
    NUInt,
    Int,
    UInt,
    Short,
    UShort,
    SByte,
    UByte,
    SysBool,
    SysChar
};
use util::NativeRef;
use std::default::Default;
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
impl Compile for f64 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_float64_constant(func.as_ptr(), jit_type_float64, *self) )
        }
    }
    #[inline]
    fn jit_type(_:Option<f64>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_float64)
        }
    }
}
impl Compile for f32 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_float32_constant(func.as_ptr(), jit_type_float32, *self) )
        }
    }
    #[inline]
    fn jit_type(_:Option<f32>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_float32)
        }
    }
}
impl Compile for int {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_long_constant(func.as_ptr(), jit_type_nint, *self as i64) )
        }
    }
    #[inline]
    fn jit_type(_:Option<int>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_nint)
        }
    }
}
impl Compile for uint {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_nuint, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<uint>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_nuint)
        }
    }
}
impl Compile for i32 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_int, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<i32>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_int)
        }
    }
}
impl Compile for u32 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_uint, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<u32>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_uint)
        }
    }
}
impl Compile for i16 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_short, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<i16>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_short)
        }
    }
}
impl Compile for u16 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_ushort, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<u16>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_ushort)
        }
    }
}
impl Compile for i8 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_sbyte, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<i8>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_sbyte)
        }
    }
}
impl Compile for u8 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_ubyte, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<u8>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_ubyte)
        }
    }
}
impl Compile for bool {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_sys_bool, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<bool>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_sys_bool)
        }
    }
}
impl Compile for char {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_sys_char, *self as jit_nint) )
        }
    }
    #[inline]
    fn jit_type(_:Option<char>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_sys_char)
        }
    }
}
impl<'t> Compile for &'t str {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let cstring_t = Compile::jit_type(None::<&'t str>);
        let strlen_i = (self.len() as i32).compile(func);
        let bufptr = Value::new(func, &cstring_t);
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
        unsafe {
            Type::create_pointer(&NativeRef::from_ptr(jit_type_sys_char))
        }
    }
}
impl Compile for String {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        self.as_slice().compile(func)
    }
    #[inline]
    fn jit_type(_:Option<String>) -> Type {
        unsafe {
            Type::create_pointer(&NativeRef::from_ptr(jit_type_sys_char))
        }
    }
}
impl<T:Compile> Compile for *T {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&self.to_uint().compile(func), &::get_type::<T>(), false)
    }
    #[inline]
    fn jit_type(_:Option<*T>) -> Type {
        Type::create_pointer(&::get_type::<T>())
    }
}
impl<R:Compile> Compile for fn() -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn() -> R).to_uint().compile(func), &::get_type::<fn() -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn() -> R>) -> Type {
        Type::create_signature(CDECL, &::get_type::<R>(), &mut [])
    }
}
impl<A:Compile, R:Compile> Compile for fn(A) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn(A) -> R).to_uint().compile(func), &::get_type::<fn(A) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A) -> R>) -> Type {
        Type::create_signature(CDECL, &::get_type::<R>(), &mut [&::get_type::<A>()])
    }
}
impl<A:Compile, B:Compile, R:Compile> Compile for fn(A, B) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn(A, B) -> R).to_uint().compile(func), &::get_type::<fn(A, B) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A, B) -> R>) -> Type {
        Type::create_signature(CDECL, &::get_type::<R>(), &mut [&::get_type::<A>(), &::get_type::<B>()])
    }
}
impl<A:Compile, B:Compile, C:Compile, R:Compile> Compile for fn(A, B, C) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        func.insn_convert(&(self as *fn(A, B, C) -> R).to_uint().compile(func), &::get_type::<fn(A, B, C) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A, B, C) -> R>) -> Type {
        Type::create_signature(CDECL, &::get_type::<R>(), &mut [&::get_type::<A>(), &::get_type::<B>(), &::get_type::<C>()])
    }
}