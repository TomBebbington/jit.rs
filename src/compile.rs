use raw::*;
use function::UncompiledFunction;
use function::ABI::CDECL;
use types::get;
use libc::c_long;
use value::Value;
use types::Type;
use util::{from_ptr, NativeRef};
/// A type that can be compiled into a LibJIT representation
pub trait Compile for Sized? {
    /// Get a JIT representation of this value
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a>;
    /// Get the JIT type repr of the value
    fn jit_type(_:Option<Self>) -> Type;
}
impl Compile for () {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        let ty = get::<()>();
        Value::new(func, ty)
    }
    #[inline(always)]
    fn jit_type(_:Option<()>) -> Type {
        unsafe {
            from_ptr(jit_type_void)
        }
    }
}
compile_prims!{
    (f64, f64) => (jit_type_float64, jit_value_create_float64_constant),
    (f32, f32) => (jit_type_float32, jit_value_create_float32_constant),
    (int, c_long) => (jit_type_nint, jit_value_create_nint_constant),
    (uint, c_long) => (jit_type_nuint, jit_value_create_nint_constant),
    (i64, c_long) => (jit_type_long, jit_value_create_long_constant),
    (u64, c_long) => (jit_type_ulong, jit_value_create_long_constant),
    (i32, c_long) => (jit_type_int, jit_value_create_nint_constant),
    (u32, c_long) => (jit_type_uint, jit_value_create_nint_constant),
    (i16, c_long) => (jit_type_short, jit_value_create_nint_constant),
    (u16, c_long) => (jit_type_ushort, jit_value_create_nint_constant),
    (i8, c_long) => (jit_type_sbyte, jit_value_create_nint_constant),
    (u8, c_long) => (jit_type_ubyte, jit_value_create_nint_constant),
    (bool, c_long) => (jit_type_sys_bool, jit_value_create_nint_constant),
    (char, c_long) => (jit_type_sys_char, jit_value_create_nint_constant)
}
impl<T:Compile> Compile for *mut T {
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                get::<*mut T>().as_ptr(),
                self.to_uint() as c_long
            ))
        }
    }
    #[inline(always)]
    fn jit_type(_:Option<*mut T>) -> Type {
        Type::create_pointer(get::<T>())
    }
}
impl<T:Compile> Compile for *const T {
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                get::<*const T>().as_ptr(),
                self.to_uint() as c_long
            ))
        }
    }
    #[inline(always)]
    fn jit_type(_:Option<*const T>) -> Type {
        Type::create_pointer(get::<T>())
    }
}
impl<T:Compile> Compile for &'static T {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                get::<&'static T>().as_ptr(),
                (*self as *const T).to_uint() as c_long
            ))
        }
    }
    #[inline(always)]
    fn jit_type(_:Option<&'static T>) -> Type {
        Type::create_pointer(get::<T>())
    }
}
compile_tuple!(A, B => a, b);
compile_tuple!(A, B, C => a, b, c);
compile_tuple!(A, B, C, D => a, b, c, d);
compile_tuple!(A, B, C, D, E => a, b, c, d, e);
compile_func!(fn() -> R, fn() -> R, extern fn() -> R);
compile_func!(fn(A) -> R, fn(A) -> R, extern fn(A) -> R);
compile_func!(fn(A, B) -> R, fn(A, B) -> R, extern fn(A, B) -> R);
compile_func!(fn(A, B, C) -> R, fn(A, B, C) -> R, extern fn(A, B, C) -> R);
compile_func!(fn(A, B, C, D) -> R, fn(A, B, C, D) -> R, extern fn(A, B, C, D) -> R);