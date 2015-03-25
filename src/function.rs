use raw::*;
use context::Builder;
use compile::Compile;
use label::Label;
use types::Ty;
use insn::Block;
use util::{self, from_ptr, NativeRef};
use value::Value;
use libc::{
    c_int,
    c_uint,
    c_void
};
use std::default::Default;
use std::fmt;
use std::ops::Index;
use std::{mem, ptr};
use std::ffi::CString;
/// A platform's application binary interface
#[repr(C)]
#[derive(Copy)]
pub enum Abi {
    /// The C application binary interface
    CDecl,
    /// The C application binary interface with variable arguments
    VarArg,
    /// A Windows application binary interface*-+
    StdCall,
    /// A Windows application binary interface
    FastCall
}
impl Default for Abi {
    fn default() -> Abi {
        Abi::CDecl
    }
}
/// Call flags to a function
pub mod flags {
    use libc::c_int;
    /// Call flags to a function
    bitflags!(
        flags CallFlags: c_int {
            /// When the function won't throw a value
            const NO_THROW = 1,
            /// When the function won't return a value
            const NO_RETURN = 2,
            /// When the function is tail-recursive
            const TAIL = 4
        }
    );
}
/// A function that can be compiled or not
pub trait Function : NativeRef {
    /// Check if this function is compiled
    fn is_compiled(&self) -> bool;
    /// Get the signature of this function
    fn get_signature(&self) -> &Ty {
        unsafe { from_ptr(jit_function_get_signature(self.as_ptr())) }
    }
}
/// Any kind of function, compiled or not
native_ref!(AnyFunction contravariant {
    _func: jit_function_t
});
impl<'a> AnyFunction<'a> {
    /// Return the compiled function if there is one
    pub fn into_compiled(self) -> Option<CompiledFunction<'a>> {
        if self.is_compiled() {
            Some(unsafe { from_ptr(self.as_ptr()) })
        } else {
            None
        }
    }
    /// Return the uncompiled function if there is one
    pub fn into_uncompiled(self) -> Option<UncompiledFunction<'a>> {
        if !self.is_compiled() {
            Some(unsafe { from_ptr(self.as_ptr()) })
        } else {
            None
        }
    }
}
impl<'a> Function for AnyFunction<'a> {
    /// Check if this function is compiled
    fn is_compiled(&self) -> bool {
        unsafe { jit_function_is_compiled(self.as_ptr()) != 0 }
    }
}
/// A function which has already been compiled from an `UncompiledFunction`. This can
/// be called but not added to.
///
/// A function persists for the lifetime of its containing context. This is
/// a function which has already been compiled and is now in executable form.
native_ref!(CompiledFunction contravariant {
    _func: jit_function_t
});
impl<'a> Function for CompiledFunction<'a> {
    /// 10/10 would compile again
    fn is_compiled(&self) -> bool {
        true
    }
}
impl<'a> fmt::Display for CompiledFunction<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", try!(util::dump(|fd| unsafe {
            jit_dump_function(mem::transmute(fd), self.as_ptr(), ptr::null());
        })))
    }
}
impl<'a> CompiledFunction<'a> {
    /// Run a closure with the compiled function as an argument
    pub fn with<A, R, F:FnOnce(extern "C" fn(A) -> R)>(&self, cb:F) {
        cb(unsafe {
            mem::transmute(jit_function_to_closure(self._func))
        })
    }
}

#[derive(PartialEq)]
/// A function which has not been compiled yet, so it can have instructions added to it.
///
/// A function persists for the lifetime of its containing context. This represents
/// the function in the "building" state, where the user constructs instructions
/// that represents the function body. Once the build process is complete, the
/// user calls `function.compile()` to convert it into its executable form.
pub struct UncompiledFunction<'a> {
    _func: jit_function_t,
    args: Vec<Value<'a>>,
    owned: bool
}
impl<'a> NativeRef for UncompiledFunction<'a> {
    #[inline(always)]
    /// Convert to a native pointer
    unsafe fn as_ptr(&self) -> jit_function_t {
        self._func
    }
    #[inline(always)]
    /// Convert from a native pointer
    unsafe fn from_ptr(ptr:jit_function_t) -> UncompiledFunction<'a> {
        let sig = jit_function_get_signature(ptr);
        println!("{} - {:?}", jit_type_num_params(sig), sig);
        let args = (0..jit_type_num_params(sig)).map(|i| from_ptr(jit_value_get_param(ptr, i as c_uint))).collect::<Vec<_>>();
        println!("{:?}", args);
        UncompiledFunction {
            _func: ptr,
            args: args,
            owned: false
        }
    }
}
impl<'a> Function for UncompiledFunction<'a> {
    fn is_compiled(&self) -> bool {
        false
    }
}
impl<'a> fmt::Display for UncompiledFunction<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", try!(util::dump(|fd| unsafe {
            jit_dump_function(mem::transmute(fd), self.as_ptr(), ptr::null());
        })))
    }
}
#[unsafe_destructor]
impl<'a> Drop for UncompiledFunction<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                jit_function_abandon(self.as_ptr());
            }
        }
    }
}
impl<'a> Index<usize> for UncompiledFunction<'a> {
    type Output = Value<'a>;
    /// Get the value that corresponds to a specified function parameter.
    fn index(&self, param: &usize) -> &Value<'a> {
        &self.args[*param]
    }
}
impl<'a> UncompiledFunction<'a> {
    #[inline(always)]
    /// Create a new function block and associate it with a JIT context.
    /// It is recommended that you call `Function::new` and `function.compile()`
    /// in the closure you give to `context.build(...)`.
    ///
    /// This will protect the JIT's internal data structures within a
    /// multi-threaded environment.
    pub fn new(context:&'a Builder, signature:&Ty) -> UncompiledFunction<'a> {
        unsafe {
            let mut me:UncompiledFunction = from_ptr(jit_function_create(
                context.as_ptr(),
                signature.as_ptr()
            ));
            me.owned = true;
            if cfg!(test) {
                me.set_recompilable();
                me.set_optimization_level(UncompiledFunction::get_max_optimization_level());
            }
            me
        }
    }
    #[inline(always)]
    /// Create a new function block and associate it with a JIT context.
    /// In addition, this function is nested inside the specified *parent*
    /// function and is able to access its parent's (and grandparent's) local
    /// variables.
    ///
    /// The front end is responsible for ensuring that the nested function can
    /// never be called by anyone except its parent and sibling functions.
    /// The front end is also responsible for ensuring that the nested function
    /// is compiled before its parent.
    pub fn new_nested(context:&'a Builder, signature: &Ty,
                        parent: &'a UncompiledFunction<'a>) -> UncompiledFunction<'a> {
        unsafe {
            let mut me:UncompiledFunction = from_ptr(jit_function_create_nested(
                context.as_ptr(),
                signature.as_ptr(),
                parent.as_ptr()
            ));
            me.owned = true;
            if cfg!(bench) {
                me.set_recompilable();
                me.set_optimization_level(UncompiledFunction::get_max_optimization_level());
            }
            me
        }
    }
    #[inline(always)]
    /// Make an instruction that converts the value to the type given
    pub fn insn_convert(&self, v: Value<'a>,
                            t:&Ty, overflow_check:bool) -> Value<'a> {
        unsafe {
            from_ptr(jit_insn_convert(
                self.as_ptr(),
                v.as_ptr(),
                t.as_ptr(),
                overflow_check as c_int
            ))
        }
    }
    #[inline(always)]
    /// Make an instructional representation of a Rust value
    pub fn insn_of<T>(&self, val:&T) -> Value<'a> where T:Compile {
        val.compile(self)
    }
    #[inline(always)]
    /// Notify the function building process that this function has a catch block
    /// in it. This must be called before any code that is part of a try block
    pub fn insn_uses_catcher(&self) {
        unsafe {
            jit_insn_uses_catcher(self.as_ptr());
        }
    }
    #[inline(always)]
    /// Throw an exception from the function with the value given
    pub fn insn_throw(&self, retval: Value<'a>) {
        unsafe {
            jit_insn_throw(self.as_ptr(), retval.as_ptr());
        }
    }
    #[inline(always)]
    /// Return from the function with the value given
    pub fn insn_return(&self, retval: Value<'a>) {
        unsafe {
            jit_insn_return(self.as_ptr(), retval.as_ptr());
        }
    }
    #[inline(always)]
    /// Return from the function
    pub fn insn_default_return(&self) {
        unsafe {
            jit_insn_default_return(self.as_ptr());
        }
    }
    #[inline(always)]
    /// Make an instruction that multiplies the values
    pub fn insn_mul(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_mul)
    }
    #[inline(always)]
    /// Make an instruction that multiplies the values and throws upon overflow
    pub fn insn_mul_ovf(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_mul_ovf)
    }
    #[inline(always)]
    /// Make an instruction that adds the values
    pub fn insn_add(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_add)
    }
    #[inline(always)]
    /// Make an instruction that adds the values and throws upon overflow
    pub fn insn_add_ovf(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_add_ovf)
    }
    #[inline(always)]
    /// Make an instruction that subtracts the second value from the first
    pub fn insn_sub(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_sub)
    }
    #[inline(always)]
    /// Make an instruction that subtracts the second value from the first and throws upon overflow
    pub fn insn_sub_ovf(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_sub_ovf)
    }
    #[inline(always)]
    /// Make an instruction that divides the first number by the second
    pub fn insn_div(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_div)
    }
    #[inline(always)]
    /// Make an instruction that finds the remainder when the first number is
    /// divided by the second
    pub fn insn_rem(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_rem)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is lower than or
    /// equal to the second
    pub fn insn_leq(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_le)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is greater than or
    /// equal to the second
    pub fn insn_geq(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_ge)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is lower than the second
    pub fn insn_lt(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_lt)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is greater than the second
    pub fn insn_gt(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_gt)
    }
    #[inline(always)]
    /// Make an instruction that checks if the values are equal
    pub fn insn_eq(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_eq)
    }
    #[inline(always)]
    /// Make an instruction that checks if the values are not equal
    pub fn insn_neq(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_ne)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise and on the two values
    pub fn insn_and(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_and)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise or on the two values
    pub fn insn_or(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_or)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise xor on the two values
    pub fn insn_xor(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_xor)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise not on the two values
    pub fn insn_not(&self, value: Value<'a>) -> Value<'a> {
        self.insn_unop(value, jit_insn_not)
    }
    #[inline(always)]
    /// Make an instruction that performs a left bitwise shift on the first
    /// value by the second value
    pub fn insn_shl(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_shl)
    }
    #[inline(always)]
    /// Make an instruction that performs a right bitwise shift on the first
    /// value by the second value
    pub fn insn_shr(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_shr)
    }
    /// Make an instruction that performs a right bitwise shift on the first
    /// value by the second value
    pub fn insn_ushr(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_ushr)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise negate on the value
    pub fn insn_neg(&self, value: Value<'a>) -> Value<'a> {
        self.insn_unop(value, jit_insn_neg)
    }
    #[inline(always)]
    /// Make an instruction that duplicates the value given
    pub fn insn_dup(&self, value: Value<'a>) -> Value<'a> {
        unsafe {
            let dup_value = jit_insn_load(self.as_ptr(), value.as_ptr());
            from_ptr(dup_value)
        }
    }
    #[inline(always)]
    /// Make an instruction that loads a value from a src value
    pub fn insn_load(&self, src: Value<'a>) -> Value<'a> {
        self.insn_unop(src, jit_insn_load)
    }
    #[inline(always)]
    /// Make an instruction that loads a value from a src value
    pub fn insn_load_relative(&self, src: Value<'a>, offset: usize, ty: &Ty) -> Value<'a> {
        unsafe {
            from_ptr(jit_insn_load_relative(
                self.as_ptr(),
                src.as_ptr(),
                offset as jit_nint,
                ty.as_ptr()
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that stores a value at a destination value
    pub fn insn_store(&self, dest: Value<'a>, src: Value<'a>) {
        unsafe {
            jit_insn_store(self.as_ptr(), dest.as_ptr(), src.as_ptr());
        }
    }
    #[inline(always)]
    /// Make an instruction that stores a value a certain offset away from a
    /// destination value
    pub fn insn_store_relative(&self, dest: Value<'a>, offset: usize,
                               src: Value<'a>) {
        unsafe {
            jit_insn_store_relative(self.as_ptr(), dest.as_ptr(), offset as jit_nint, src.as_ptr());
        }
    }
    #[inline(always)]
    /// Make an instruction that sets a label
    pub fn insn_label(&self, label: &mut Label<'a>) {
        unsafe {
            jit_insn_label(self.as_ptr(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label
    pub fn insn_branch(&self, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch(self.as_ptr(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label if the value is true
    pub fn insn_branch_if(&self, value: Value<'a>, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch_if(self.as_ptr(), value.as_ptr(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label if the value is false
    pub fn insn_branch_if_not(&self, value: Value<'a>, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch_if_not(self.as_ptr(), value.as_ptr(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a label in the table
    pub fn insn_jump_table(&self, value: Value<'a>, labels: &mut [Label<'a>]) {
        unsafe {
            let mut native_labels: Vec<_> = labels.iter()
                .map(|label| **label).collect();
            jit_insn_jump_table(
                self.as_ptr(),
                value.as_ptr(),
                native_labels.as_mut_ptr(),
                labels.len() as c_uint
            );
        }
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse cosine of the number given
    pub fn insn_acos(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_acos)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse sine of the number given
    pub fn insn_asin(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_asin)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse tangent of the number given
    pub fn insn_atan(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_atan)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse tangent of the numbers given
    pub fn insn_atan2(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_atan2)
    }
    #[inline(always)]
    /// Make an instruction that finds the nearest integer above a number
    pub fn insn_ceil(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_ceil)
    }
    #[inline(always)]
    /// Make an instruction that gets the consine of the number given
    pub fn insn_cos(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_cos)
    }
    #[inline(always)]
    /// Make an instruction that gets the hyperbolic consine of the number given
    pub fn insn_cosh(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_cosh)
    }
    #[inline(always)]
    /// Make an instruction that gets the natural logarithm rased to the power
    /// of the number
    pub fn insn_exp(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_exp)
    }
    #[inline(always)]
    /// Make an instruction that finds the nearest integer below a number
    pub fn insn_floor(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_floor)
    }
    #[inline(always)]
    /// Make an instruction that gets the natural logarithm of the number
    pub fn insn_log(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_log)
    }
    #[inline(always)]
    /// Make an instruction that gets the base 10 logarithm of the number
    pub fn insn_log10(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_log10)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of raising the first value to
    /// the power of the second value
    pub fn insn_pow(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_pow)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of rounding the value to the
    /// nearest integer
    pub fn insn_rint(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_rint)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of rounding the value to the
    /// nearest integer
    pub fn insn_round(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_round)
    }
    #[inline(always)]
    /// Make an instruction the gets the sine of the number
    pub fn insn_sin(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sin)
    }
    #[inline(always)]
    /// Make an instruction the gets the hyperbolic sine of the number
    pub fn insn_sinh(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sinh)
    }
    #[inline(always)]
    /// Make an instruction the gets the square root of a number
    pub fn insn_sqrt(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sqrt)
    }
    #[inline(always)]
    /// Make an instruction the gets the tangent of a number
    pub fn insn_tan(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_tan)
    }
    #[inline(always)]
    /// Make an instruction the gets the hyperbolic tangent of a number
    pub fn insn_tanh(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_tanh)
    }
    #[inline(always)]
    /// Make an instruction that truncates the value
    pub fn insn_trunc(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_trunc)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is NaN
    pub fn insn_is_nan(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_is_nan)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is finite
    pub fn insn_is_finite(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_is_finite)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is  infinite
    pub fn insn_is_inf(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_is_inf)
    }
    #[inline(always)]
    /// Make an instruction that gets the absolute value of a number
    pub fn insn_abs(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_abs)
    }
    #[inline(always)]
    /// Make an instruction that gets the smallest of two numbers
    pub fn insn_min(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_min)
    }
    #[inline(always)]
    /// Make an instruction that gets the biggest of two numbers
    pub fn insn_max(&self, v1: Value<'a>, v2: Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_max)
    }
    #[inline(always)]
    /// Make an instruction that gets the sign of a number
    pub fn insn_sign(&self, v: Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sign)
    }

    /// Call the function, which may or may not be translated yet
    pub fn insn_call<F>(&self, name:Option<&str>, func:&F, sig:Option<&Ty>,
        args: &mut [Value<'a>], flags: flags::CallFlags) -> Value<'a> where F:Function {
        unsafe {
            let mut native_args:Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();
            let c_name = name.map(|name| CString::new(name.as_bytes()).unwrap());
            from_ptr(jit_insn_call(
                self.as_ptr(),
                c_name.map(|name| mem::transmute(name.as_ptr())).unwrap_or(ptr::null_mut()),
                func.as_ptr(), sig.as_ptr(), native_args.as_mut_ptr(),
                args.len() as c_uint,
                flags.bits()
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that calls a function that has the signature given
    /// with some arguments through a pointer to the fucntion
    pub fn insn_call_indirect(&self, func:Value<'a>, signature: &Ty,
                               args: &mut [Value<'a>], flags: flags::CallFlags) -> Value<'a> {
        unsafe {
            let mut native_args:Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();
            from_ptr(jit_insn_call_indirect(self.as_ptr(), func.as_ptr(), signature.as_ptr(), native_args.as_mut_ptr(), args.len() as c_uint, flags.bits() as c_int))
        }
    }
    /// Make an instruction that calls a native function that has the signature
    /// given with some arguments
    fn insn_call_native(&self, name: Option<&str>,
                        native_func: *mut c_void, signature: &Ty,
                        args: &mut [Value<'a>], flags: flags::CallFlags) -> Value<'a> {
        unsafe {
            let mut native_args:Vec<_> = args.iter()
                .map(|arg| arg.as_ptr()).collect();
            let c_name = name.map(|name| CString::new(name.as_bytes()).unwrap());
            from_ptr(jit_insn_call_native(
                self.as_ptr(),
                c_name.map(|name| mem::transmute(name.as_ptr())).unwrap_or(ptr::null_mut()),
                native_func,
                signature.as_ptr(),
                native_args.as_mut_ptr(),
                args.len() as c_uint,
                flags.bits()
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with no arguments and expects a return value
    pub fn insn_call_native0<R>(&self, name: Option<&str>,
                            native_func: extern fn() -> R,
                            signature: &Ty,
                            flags: flags::CallFlags) -> Value<'a> {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, [].as_mut_slice(), flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with a single argument and expects a return value
    pub fn insn_call_native1<A,R>(&self, name: Option<&str>,
                                native_func: extern fn(A) -> R,
                                signature: &Ty,
                                mut args: [Value<'a>; 1],
                                flags: flags::CallFlags) -> Value<'a> {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice(), flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with two arguments and expects a return value
    pub fn insn_call_native2<A,B,R>(&self, name: Option<&str>,
                                native_func: extern fn(A, B) -> R,
                                signature: &Ty,
                                mut args: [Value<'a>; 2],
                                flags: flags::CallFlags) -> Value<'a> {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice(), flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with three arguments and expects a return value
    pub fn insn_call_native3<A,B,C,R>(&self, name: Option<&str>,
                                native_func: extern fn(A, B, C) -> R,
                                signature: &Ty,
                                mut args: [Value<'a>; 3],
                                flags: flags::CallFlags) -> Value<'a> {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice(), flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with four arguments and expects a return value
    pub fn insn_call_native4<A,B,C,D,R>(&self, name: Option<&str>,
                                native_func: extern fn(A, B, C, D) -> R,
                                signature: &Ty,
                                mut args: [Value<'a>; 4],
                                flags: flags::CallFlags) -> Value<'a> {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice()
            , flags)
    }
    #[inline(always)]
    /// Make an instruction that copies memory from a source address to a destination address
    pub fn insn_memcpy(&self, dest: Value<'a>, source: Value<'a>, size: Value<'a>) -> bool {
        unsafe {
            jit_insn_memcpy(self.as_ptr(), dest.as_ptr(), source.as_ptr(), size.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Make an instruction that moves memory from a source address to a destination address
    pub fn insn_memmove(&self, dest: Value<'a>, source: Value<'a>, size: Value<'a>) -> bool {
        unsafe {
            jit_insn_memmove(self.as_ptr(), dest.as_ptr(), source.as_ptr(), size.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Make an instruction that sets memory at the destination address
    pub fn insn_memset(&self, dest: Value<'a>, source: Value<'a>, size: Value<'a>) -> bool {
        unsafe {
            jit_insn_memset(self.as_ptr(), dest.as_ptr(), source.as_ptr(), size.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Make an instruction that allocates some space
    pub fn insn_alloca(&self, size: Value<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_insn_alloca(self.as_ptr(), size.as_ptr()))
        }
    }
    #[inline(always)]
    /// Make an instruction that gets the address of a value
    pub fn insn_address_of(&self, value: Value<'a>) -> Value<'a> {
        unsafe {
            from_ptr(jit_insn_address_of(self.as_ptr(), value.as_ptr()))
        }
    }
    #[inline(always)]
    fn insn_binop(&self,
                    v1: Value<'a>, v2: Value<'a>,
                    f: unsafe extern "C" fn(
                        jit_function_t,
                        jit_value_t,
                        jit_value_t) -> jit_value_t)
                    -> Value<'a> {
        unsafe {
            from_ptr(f(self.as_ptr(), v1.as_ptr(), v2.as_ptr()))
        }
    }
    #[inline(always)]
    fn insn_unop(&self,
                    value: Value<'a>,
                    f: unsafe extern "C" fn(
                        jit_function_t,
                        jit_value_t) -> jit_value_t)
                    -> Value<'a> {
        unsafe {
            from_ptr(f(self.as_ptr(), value.as_ptr()))
        }
    }
    #[inline(always)]
    /// Make instructions to run the block if the condition is met
    pub fn insn_if<B>(&self, cond: Value<'a>, block: B) where B:FnOnce() {
        let mut after = Label::new(self);
        self.insn_branch_if_not(cond, &mut after);
        block();
        self.insn_label(&mut after);
    }
    #[inline(always)]
    /// Make instructions to run the block if the condition is not met
    pub fn insn_if_not<B>(&self, cond: Value<'a>, block: B) where B:FnOnce() {
        let mut after = Label::new(self);
        self.insn_branch_if(cond, &mut after);
        block();
        self.insn_label(&mut after);
    }
    /// Make instructions to run the block forever
    pub fn insn_loop<B>(&self, block: B) where B:FnOnce() {
        let mut start = Label::new(self);
        self.insn_label(&mut start);
        block();
        self.insn_branch(&mut start);
    }
    /// Make instructions to run the block and continue running it so long
    /// as the condition is met
    pub fn insn_loop_while<C, B>(&self, cond: C, block: B)
        where C:FnOnce() -> Value<'a>, B:FnOnce()  {
        let mut start = Label::new(self);
        self.insn_label(&mut start);
        block();
        self.insn_branch_if(cond(), &mut start);
    }
    /// Make instructions to run the block and continue running it so long
    /// as the condition is met
    pub fn insn_while<C, B>(&self, cond: C, block: B)
        where C:FnOnce() -> Value<'a>, B:FnOnce() {
        let mut start = Label::new(self);
        self.insn_label(&mut start);
        let mut after = Label::new(self);
        let cond_v = cond();
        self.insn_branch_if_not(cond_v, &mut after);
        block();
        self.insn_branch(&mut start);
        self.insn_label(&mut after);
    }
    #[inline(always)]
    /// Set the optimization level of the function, where the bigger the level,
    /// the more effort should be spent optimising
    pub fn set_optimization_level(&self, level: c_uint) {
        unsafe {
            jit_function_set_optimization_level(self.as_ptr(), level);
        }
    }
    #[inline(always)]
    /// Get the max optimization level
    pub fn get_max_optimization_level() -> c_uint {
        unsafe {
            jit_function_get_max_optimization_level()
        }
    }
    #[inline(always)]
    /// Make this function a candidate for recompilation
    pub fn set_recompilable(&self) {
        unsafe {
            jit_function_set_recompilable(self.as_ptr());
        }
    }
    /// Get the entry block of this function
    pub fn get_entry(&self) -> Option<Block<'a>> {
        unsafe {
            from_ptr(jit_function_get_entry(self.as_ptr()))
        }
    }
    /// Get the current block of this function
    pub fn get_current(&self) -> Option<Block<'a>> {
        unsafe {
            from_ptr(jit_function_get_current(self.as_ptr()))
        }
    }
    #[inline(always)]
    /// Compile the function
    pub fn compile(self) -> CompiledFunction<'a> {
        if !self.owned {
            panic!("The function must be owned")
        }
        unsafe {
            let ptr = self.as_ptr();
            mem::forget(self);
            jit_function_compile(ptr);
            from_ptr(ptr)
        }
    }
    #[inline(always)]
    /// Compile the function into a closure directly
    pub fn compile_with<A, R, F>(self, cb: F) -> CompiledFunction<'a>
        where F:FnOnce(extern fn(A) -> R) {
        let compiled = self.compile();
        compiled.with(cb);
        compiled
    }
}
