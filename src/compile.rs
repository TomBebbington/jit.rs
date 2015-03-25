use raw::*;
use function::UncompiledFunction;
use function::Abi::CDecl;
use types::get;
use libc::c_long;
use value::Value;
use types::{consts, CowType, Type};
use util::{from_ptr, NativeRef};
use std::borrow::IntoCow;
use std::ffi::CString;
/// A type that can be compiled into a LibJIT representation
pub trait Compile {
    /// Get a JIT representation of this value
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a>;
    /// Get the type descriptor that represents this type
    fn get_type() -> CowType<'static>;
}
impl Compile for () {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        Value::new(func, consts::get_void())
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        consts::get_void().into_cow()
    }
}
compile_prims!{
    (f64, f64) => (get_float64, jit_value_create_float64_constant),
    (f32, f32) => (get_float32, jit_value_create_float32_constant),
    (isize, c_long) => (get_nint, jit_value_create_nint_constant),
    (usize, c_long) => (get_nuint, jit_value_create_nint_constant),
    (i64, c_long) => (get_long, jit_value_create_long_constant),
    (u64, c_long) => (get_ulong, jit_value_create_long_constant),
    (i32, c_long) => (get_int, jit_value_create_nint_constant),
    (u32, c_long) => (get_uint, jit_value_create_nint_constant),
    (i16, c_long) => (get_short, jit_value_create_nint_constant),
    (u16, c_long) => (get_ushort, jit_value_create_nint_constant),
    (i8, c_long) => (get_sbyte, jit_value_create_nint_constant),
    (u8, c_long) => (get_ubyte, jit_value_create_nint_constant),
    (bool, c_long) => (get_sys_bool, jit_value_create_nint_constant),
    (char, c_long) => (get_sys_char, jit_value_create_nint_constant)
}
impl<T> Compile for *mut T where T:Compile {
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                (&*get::<*mut T>()).as_ptr(),
                *self as c_long
            ))
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        Type::new_pointer(&*get::<T>()).into_cow()
    }
}
impl<T> Compile for *const T where T:Compile {
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                (&*get::<*const T>()).as_ptr(),
                *self as c_long
            ))
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        Type::new_pointer(&*get::<T>()).into_cow()
    }
}
impl<T> Compile for &'static T where T:Compile {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                (&*get::<&'static T>()).as_ptr(),
                *self as *const T as c_long
            ))
        }
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        Type::new_pointer(&get::<T>()).into_cow()
    }
}
impl Compile for CString {
    #[inline(always)]
    fn compile<'a>(&self, func:&UncompiledFunction<'a>) -> Value<'a> {
        self.as_ptr().compile(func)
    }
    #[inline(always)]
    fn get_type() -> CowType<'static> {
        Type::new_pointer(consts::get_sys_char()).into_cow()
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
