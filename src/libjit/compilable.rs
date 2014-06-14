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
pub trait Compilable {
	/// Get a JIT representation of this value
	fn compile(&self, func:&Function) -> Value;
	/// Get the JIT type repr of the value
	fn jit_type(_:Option<Self>) -> Type;
}
#[cfg(test)]
fn test_compile<T:Compilable+Default>(kind:TypeKind) {
	::with_empty_func(|_, func| {
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
	#[inline]
	fn jit_type(_:Option<()>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_void_ptr)
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
	#[inline]
	fn jit_type(_:Option<f64>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_float64)
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
	#[inline]
	fn jit_type(_:Option<f32>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_float32)
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
	#[inline]
	fn jit_type(_:Option<int>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_nint)
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
	#[inline]
	fn jit_type(_:Option<uint>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_nuint)
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
	#[inline]
	fn jit_type(_:Option<i32>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_int)
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
	#[inline]
	fn jit_type(_:Option<u32>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_uint)
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
	#[inline]
	fn jit_type(_:Option<i16>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_short)
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
	#[inline]
	fn jit_type(_:Option<u16>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_ushort)
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
	#[inline]
	fn jit_type(_:Option<i8>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_sbyte)
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
	#[inline]
	fn jit_type(_:Option<u8>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_ubyte)
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
	#[inline]
	fn jit_type(_:Option<bool>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_sys_bool)
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
	#[inline]
	fn jit_type(_:Option<char>) -> Type {
		unsafe {
			NativeRef::from_ptr(jit_type_sys_char)
		}
	}
}
#[test]
fn test_compile_char() {
	test_compile::<char>(SysChar)
}
impl<'t> Compilable for &'t str {
	fn compile(&self, func:&Function) -> Value {
		let cstring_t = Compilable::jit_type(None::<&'t str>);
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
impl Compilable for String {
	fn compile(&self, func:&Function) -> Value {
		self.as_slice().compile(func)
	}
	#[inline]
	fn jit_type(_:Option<String>) -> Type {
		unsafe {
			Type::create_pointer(&NativeRef::from_ptr(jit_type_sys_char))
		}
	}
}
impl<T:Compilable> Compilable for *T {
	fn compile(&self, func:&Function) -> Value {
		func.insn_convert(&self.to_uint().compile(func), &::get_type::<T>(), false)
	}
	#[inline]
	fn jit_type(_:Option<*T>) -> Type {
		unsafe {
			Type::create_pointer(&::get_type::<T>())
		}
	}
}
impl<R:Compilable> Compilable for fn() -> R {
	fn compile(&self, func:&Function) -> Value {
		func.insn_convert(&(self as *fn() -> R).to_uint().compile(func), &::get_type::<fn() -> R>(), false)
	}
	#[inline]
	fn jit_type(_:Option<fn() -> R>) -> Type {
		Type::create_signature(CDECL, &::get_type::<R>(), &mut [])
	}
}
impl<A:Compilable, R:Compilable> Compilable for fn(A) -> R {
	fn compile(&self, func:&Function) -> Value {
		func.insn_convert(&(self as *fn(A) -> R).to_uint().compile(func), &::get_type::<fn(A) -> R>(), false)
	}
	#[inline]
	fn jit_type(_:Option<fn(A) -> R>) -> Type {
		Type::create_signature(CDECL, &::get_type::<R>(), &mut [&::get_type::<A>()])
	}
}
impl<A:Compilable, B:Compilable, R:Compilable> Compilable for fn(A, B) -> R {
	fn compile(&self, func:&Function) -> Value {
		func.insn_convert(&(self as *fn(A, B) -> R).to_uint().compile(func), &::get_type::<fn(A, B) -> R>(), false)
	}
	#[inline]
	fn jit_type(_:Option<fn(A, B) -> R>) -> Type {
		Type::create_signature(CDECL, &::get_type::<R>(), &mut [&::get_type::<A>(), &::get_type::<B>()])
	}
}
impl<A:Compilable, B:Compilable, C:Compilable, R:Compilable> Compilable for fn(A, B, C) -> R {
	fn compile(&self, func:&Function) -> Value {
		func.insn_convert(&(self as *fn(A, B, C) -> R).to_uint().compile(func), &::get_type::<fn(A, B, C) -> R>(), false)
	}
	#[inline]
	fn jit_type(_:Option<fn(A, B, C) -> R>) -> Type {
		Type::create_signature(CDECL, &::get_type::<R>(), &mut [&::get_type::<A>(), &::get_type::<B>(), &::get_type::<C>()])
	}
}