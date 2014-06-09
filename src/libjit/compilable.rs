use bindings::*;
use function::Function;
use value::Value;
use types::*;
use util::NativeRef;
/// A type that can be compiled into a LibJIT representation
pub trait Compilable {
	/// Get a JIT representation of this value
	fn compile(&self, func:&Function) -> Value;
}
#[cfg(test)]
fn test_compile<T:Compilable+Default>(kind:TypeKind) {
	with_empty_func(|_, func| {
		let pval:T = Default::default();
		let val = pval.compile(func);
		assert!(val.get_type().get_kind().contains(kind));
	})
}
impl Compilable for () {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_void_ptr, 0))
		}
	}
}
#[test]
fn test_compile_unit() {
	test_compile::<()>(Void)
}
impl Compilable for f64 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_float64_constant(func.as_ptr(), jit_type_float64, *self) )
		}
	}
}
#[test]
fn test_compile_f64() {
	test_compile::<f64>(Float64)
}
impl Compilable for f32 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_float32_constant(func.as_ptr(), jit_type_float32, *self) )
		}
	}
}
#[test]
fn test_compile_f32() {
	test_compile::<f32>(Float32)
}
impl Compilable for int {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_long_constant(func.as_ptr(), jit_type_nint, *self as i64) )
		}
	}
}
#[test]
fn test_compile_int() {
	test_compile::<int>(NInt)
}
impl Compilable for uint {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_nuint, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_uint() {
	test_compile::<uint>(NUInt)
}
impl Compilable for i32 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_int, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_i32() {
	test_compile::<i32>(Int)
}
impl Compilable for u32 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_uint, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_u32() {
	test_compile::<u32>(UInt)
}
impl Compilable for i16 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_short, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_i16() {
	test_compile::<i16>(Short)
}
impl Compilable for u16 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_ushort, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_u16() {
	test_compile::<u16>(UShort)
}
impl Compilable for i8 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_sbyte, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_i8() {
	test_compile::<i8>(SByte)
}
impl Compilable for u8 {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_ubyte, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_u8() {
	test_compile::<u8>(UByte)
}
impl Compilable for bool {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_sys_bool, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_bool() {
	test_compile::<bool>(SysBool)
}
impl Compilable for char {
	fn compile(&self, func:&Function) -> Value {
		unsafe {
			NativeRef::from_ptr(jit_value_create_nint_constant(func.as_ptr(), jit_type_sys_char, *self as jit_nint) )
		}
	}
}
#[test]
fn test_compile_char() {
	test_compile::<char>(SysChar)
}
impl<'t> Compilable for &'t str {
	fn compile(&self, func:&Function) -> Value {
		let cstring_t = Types::get_cstring();
		let strlen_i = (self.len() as i32).compile(func);
		let bufptr = func.create_value(&cstring_t);
		func.insn_store(&bufptr, &func.insn_alloca(&strlen_i));
		for i in range(0, self.len()) {
			let char_i = self.char_at(i).compile(func);
			func.insn_store_relative(&bufptr, i as int, &char_i);
		}
		let null_term = '\0'.compile(func);
		func.insn_store_relative(&bufptr, self.len() as int, &null_term);
		bufptr
	}
}
impl Compilable for String {
	fn compile(&self, func:&Function) -> Value {
		self.as_slice().compile(func)
	}
}