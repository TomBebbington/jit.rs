use function::{CDECL, Function};
use context::Context;
use types::{Type, Types};
use libc::c_void;
/// A structure that wraps a native object
pub trait NativeRef {
	/// Returns the native reference encapsulated by this object
	unsafe fn as_ptr(&self) -> *mut c_void;
	/// Returns a wrapped version of the native reference given, even if the reference is null
	unsafe fn from_ptr(ptr:*mut c_void) -> Self;
	#[inline]
	/// Returns a wrapped version of the native reference given, and check if the reference is null
	unsafe fn from_ptr_opt(ptr:*mut c_void) -> Option<Self> {
		if ptr.is_null() {
			None
		} else {
			Some(NativeRef::from_ptr(ptr))
		}
	}
}
#[cfg(test)]
pub fn with_empty_func(cb:|&Context, &Function| -> ()) -> () {
	let ctx = Context::new();
	ctx.build(|| {
		let sig = Type::create_signature(CDECL, &Types::get_void(), &mut[]);
		let func = Function::new(&ctx, &sig);
		cb(&ctx, &func)
	})
}