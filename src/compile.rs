use raw::*;
use function::UncompiledFunction;
use function::Abi::CDecl;
use types::get;
use libc::c_long;
use value::Value;
use types::{consts, StaticType, Type};
use util::{from_ptr, NativeRef};
use std::ffi::CString;
/// A type that can be compiled into a LibJIT representation
pub trait Compile {
    /// Get a JIT representation of this value
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a>;
    /// Get the type descriptor that represents this type
    fn get_type() -> Type;
}
impl Compile for () {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        let ty = get::<()>();
        Value::new(func, ty)
    }
    #[inline(always)]
    fn get_type() -> Type {
        unsafe {
            from_ptr(jit_type_void)
        }
    }
}
compile_prims!{
    (f64, f64) => (FLOAT64, jit_value_create_float64_constant),
    (f32, f32) => (FLOAT32, jit_value_create_float32_constant),
    (isize, c_long) => (NINT, jit_value_create_nint_constant),
    (usize, c_long) => (NUINT, jit_value_create_nint_constant),
    (i64, c_long) => (LONG, jit_value_create_long_constant),
    (u64, c_long) => (ULONG, jit_value_create_long_constant),
    (i32, c_long) => (INT, jit_value_create_nint_constant),
    (u32, c_long) => (UINT, jit_value_create_nint_constant),
    (i16, c_long) => (SHORT, jit_value_create_nint_constant),
    (u16, c_long) => (USHORT, jit_value_create_nint_constant),
    (i8, c_long) => (SBYTE, jit_value_create_nint_constant),
    (u8, c_long) => (UBYTE, jit_value_create_nint_constant),
    (bool, c_long) => (SYS_BOOL, jit_value_create_nint_constant),
    (char, c_long) => (SYS_CHAR, jit_value_create_nint_constant)
}
impl<T:Compile> Compile for *mut T {
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                get::<*mut T>().as_ptr(),
                *self as c_long
            ))
        }
    }
    #[inline(always)]
    fn get_type() -> Type {
        Type::new_pointer(get::<T>())
    }
}
impl<T:Compile> Compile for *const T {
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                get::<*const T>().as_ptr(),
                *self as c_long
            ))
        }
    }
    #[inline(always)]
    fn get_type() -> Type {
        Type::new_pointer(get::<T>())
    }
}
impl<T:Compile> Compile for &'static T {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                get::<&'static T>().as_ptr(),
                *self as *const T as c_long
            ))
        }
    }
    #[inline(always)]
    fn get_type() -> Type {
        Type::new_pointer(get::<T>())
    }
}
impl Compile for CString {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        self.as_ptr().compile(func)
    }
    #[inline(always)]
    fn get_type() -> Type {
        Type::new_pointer(consts::SYS_CHAR.get())
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