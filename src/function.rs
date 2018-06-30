use raw::*;
use context::Context;
use compile::Compile;
use label::Label;
use types::Ty;
use insn::Block;
use value::Val;
use util::{self, from_ptr, from_ptr_opt, from_ptr_oom};
use libc::{
    c_char,
    c_int,
    c_uint,
    c_void
};
use std::default::Default;
use std::fmt;
use std::ops::{Deref, DerefMut, Index};
use std::{mem, ptr};
use std::ffi::CString;
use std::marker::PhantomData;
/// A platform's application binary interface
///
/// This describes how the function should be called
#[repr(C)]
#[derive(Clone, Copy)]
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
    bitflags! {
        pub struct CallFlags: c_int {
            /// When the function won't throw a value
            const NO_THROW = 1;
            /// When the function won't return a value
            const NO_RETURN = 2;
            /// When the function is tail-recursive
            const TAIL = 4;
        }
    }
}
/// A function
pub struct Func(PhantomData<[()]>);
native_ref!(&Func = jit_function_t);
impl Func {
    /// Check if the given function has been compiled
    pub fn is_compiled(&self) -> bool {
        unsafe { jit_function_is_compiled(self.into()) != 0 }
    }
    /// Get the signature of the given function
    pub fn get_signature(&self) -> &Ty {
        unsafe { from_ptr(jit_function_get_signature(self.into())) }
    }
}
/// A function which has already been compiled from an `UncompiledFunction`, so it can
/// be called but not added to.
///
/// A function persists for the lifetime of its containing context. This is
/// a function which has already been compiled and is now in executable form.
#[derive(Clone, Copy)]
pub struct CompiledFunction<'a> {
    _func: jit_function_t,
    marker: PhantomData<&'a ()>
}
native_ref!(contra CompiledFunction, _func: jit_function_t);
impl<'a> Deref for CompiledFunction<'a> {
    type Target = Func;
    fn deref(&self) -> &Func {
        unsafe { mem::transmute(self._func) }
    }
}
impl<'a> DerefMut for CompiledFunction<'a> {
    fn deref_mut(&mut self) -> &mut Func {
        unsafe { mem::transmute(self._func) }
    }
}
impl<'a> fmt::Debug for CompiledFunction<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", try!(util::dump(|fd| unsafe {
            jit_dump_function(mem::transmute(fd), self.into(), ptr::null());
        })))
    }
}
impl<'a> CompiledFunction<'a> {
    /// Run a closure with the compiled function as an argument
    pub fn with<A, R, F:FnOnce(extern "C" fn(A) -> R)>(self, cb:F) {
        cb(unsafe {
            mem::transmute(jit_function_to_closure(self._func))
        })
    }
}

macro_rules! expect(
    ($name:ident, $value:expr, float) => (
        if cfg!(not(ndebug)) {
            let ty = $value.get_type();
            if !ty.is_float() {
                panic!("Value given to {} should be float, got {:?}", stringify!($name), ty);
            }
        }
    );
    ($name:ident, $value:expr, primitive) => (
        if cfg!(not(ndebug)) {
            let ty = $value.get_type();
            if !ty.is_primitive() {
                panic!("Value given to {} should be primitive, got {:?}", stringify!($name), ty);
            }
        }
    );
    ($name:ident, $v1:expr, $v2:expr, primitive) => (
        if cfg!(not(ndebug)) {
            let ty1 = $v1.get_type();
            let ty2 = $v2.get_type();
            if !ty1.is_primitive() {
                panic!("Values given to {} should be primitive, got {:?}", stringify!($name), ty1);
            } else if !ty2.is_primitive() {
                panic!("Values given to {} should be primitive, got {:?}", stringify!($name), ty2);
            }
        }
    );
    ($name:ident, $value:expr, int) => (
        if cfg!(not(ndebug)) {
            let ty = $value.get_type();
            if !ty.is_int() {
                panic!("Value given to {} should be integer, got {:?}", stringify!($name), ty);
            }
        }
    );
    ($name:ident, $dest:expr, $source:expr, $size:expr) => (
        if cfg!(not(ndebug)) {
            let dest_t = $dest.get_type();
            let source_t = $source.get_type();
            let size_t = $size.get_type();
            if !size_t.is_int() {
                panic!("Expected integer size for {}, but got {:?}", stringify!($name), size_t);
            } else if !dest_t.is_pointer() {
                panic!("Expected pointer destination for {}, but got {:?}", stringify!($name), size_t);
            } else if !source_t.is_pointer() {
                panic!("Expected pointer source for {}, but got {:?}", stringify!($name), size_t);
            }
        }
    )
);

#[derive(PartialEq)]
/// A function which has not been compiled yet, so it can have instructions added to it.
///
/// A function persists for the lifetime of its containing context. This represents
/// the function in the "building" state, where the user constructs instructions
/// that represents the function body. Once the build process is complete, the
/// user calls `function.compile()` to convert it into its executable form.
pub struct UncompiledFunction<'a> {
    _func: jit_function_t,
    marker: PhantomData<&'a ()>,
    owned: bool
}
impl<'a, 'b> From<&'a UncompiledFunction<'b>> for jit_function_t {
    /// Convert to a native pointer
    fn from(func: &'a UncompiledFunction<'b>) -> jit_function_t {
        func._func
    }
}
impl<'a, 'b> From<&'a mut UncompiledFunction<'b>> for jit_function_t {
    /// Convert to a native pointer
    fn from(func: &'a mut UncompiledFunction<'b>) -> jit_function_t {
        func._func
    }
}
impl<'a> From<UncompiledFunction<'a>> for jit_function_t {
    /// Convert to a native pointer
    fn from(func: UncompiledFunction<'a>) -> jit_function_t {
        func._func
    }
}
impl<'a> From<jit_function_t> for UncompiledFunction<'a> {
    fn from(ptr: jit_function_t) -> UncompiledFunction<'a> {
        UncompiledFunction {
            _func: ptr,
            marker: PhantomData,
            owned: false
        }
    }
}
impl<'a> fmt::Debug for UncompiledFunction<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", try!(util::dump(|fd| unsafe {
            jit_dump_function(mem::transmute(fd), self.into(), ptr::null());
        })))
    }
}

impl<'a> Deref for UncompiledFunction<'a> {
    type Target = Func;
    fn deref(&self) -> &Func {
        unsafe { mem::transmute(self._func) }
    }
}
impl<'a> DerefMut for UncompiledFunction<'a> {
    fn deref_mut(&mut self) -> &mut Func {
        unsafe { mem::transmute(self._func) }
    }
}

impl<'a> Drop for UncompiledFunction<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                jit_function_abandon(self.into());
            }
        }
    }
}
impl<'a> Index<usize> for UncompiledFunction<'a> {
    type Output = Val;
    /// Get the value that corresponds to a specified function parameter.
    fn index(&self, param: usize) -> &Val {
        let ptr = unsafe { jit_value_get_param(self.into(), param as u32) };
        if let Some(val) = from_ptr_opt(ptr) {
            val
        } else {
            panic!("Function {:?} has no parameter {}", self, param)
        }
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
    ///
    /// ```rust
    /// use jit::*;
    /// let mut ctx = Context::<()>::new();
    /// let func = UncompiledFunction::new(&mut ctx, &get::<fn(f64) -> f64>());
    /// ```
    pub fn new<T>(context:&'a mut Context<T>, signature:&Ty) -> UncompiledFunction<'a> {
        unsafe {
            let mut me:UncompiledFunction = from_ptr_oom(jit_function_create(
                context.into(),
                signature.into()
            ));
            me.owned = true;
            if cfg!(any(test, bench, ndebug)) {
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
    pub fn new_nested<T>(context:&'a mut Context<T>, signature: &Ty,
                        parent: &'a UncompiledFunction<'a>) -> UncompiledFunction<'a> {
        unsafe {
            let mut me:UncompiledFunction = from_ptr_oom(jit_function_create_nested(
                context.into(),
                signature.into(),
                parent.into()
            ));
            me.owned = true;
            if cfg!(any(test, bench, ndebug)) {
                me.set_recompilable();
                me.set_optimization_level(UncompiledFunction::get_max_optimization_level());
            }
            me
        }
    }
    #[inline(always)]
    /// Make an instruction that converts the value to the type given
    pub fn insn_convert(&self, v: &'a Val,
                            t:&Ty, overflow_check:bool) -> &'a Val {
        unsafe {
            from_ptr(jit_insn_convert(
                self.into(),
                v.into(),
                t.into(),
                overflow_check as c_int
            ))
        }
    }
    #[inline(always)]
    /// Make an instructional representation of a Rust value
    /// ```rust
    /// use jit::*;
    /// let mut ctx = Context::<()>::new();
    /// let func = UncompiledFunction::new(&mut ctx, &get::<fn() -> i32>());
    /// func.insn_return(func.insn_of(42i32));
    /// ```
    pub fn insn_of<T>(&self, val:T) -> &'a Val where T:Compile<'a> {
        val.compile(self)
    }
    #[inline(always)]
    /// Notify the function building process that this function has a catch block
    /// in it. This must be called before any code that is part of a try block
    pub fn insn_uses_catcher(&self) {
        unsafe {
            jit_insn_uses_catcher(self.into());
        }
    }
    #[inline(always)]
    /// Make an instruction to throw an exception from the function with the value given
    pub fn insn_throw(&self, retval: &'a Val) {
        unsafe {
            jit_insn_throw(self.into(), retval.into());
        }
    }
    #[inline(always)]
    /// Make an instruction that will return from the function with the value given
    pub fn insn_return(&self, retval: &'a Val) {
        unsafe {
            jit_insn_return(self.into(), retval.into());
        }
    }
    #[inline(always)]
    /// Return from the function
    pub fn insn_default_return(&self) {
        unsafe {
            jit_insn_default_return(self.into());
        }
    }
    #[inline(always)]
    /// Make an instruction that multiplies the values
    pub fn insn_mul(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_mul)
    }
    #[inline(always)]
    /// Make an instruction that multiplies the values and throws upon overflow
    pub fn insn_mul_ovf(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_mul_ovf)
    }
    #[inline(always)]
    /// Make an instruction that adds the values
    ///
    /// You can also just use `v1 + v2` in your code instead of running this method,
    /// `&Val` has the `Add` trait implemented so it can be added with normal operators.
    pub fn insn_add(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_add)
    }
    #[inline(always)]
    /// Make an instruction that adds the values and throws upon overflow
    pub fn insn_add_ovf(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_add_ovf)
    }
    #[inline(always)]
    /// Make an instruction that subtracts the second value from the first
    ///
    /// You can also just use `v1 - v2` in your code instead of running this method,
    /// `&Val` has the `Sub` trait implemented so it can be subtracted with normal operators.
    pub fn insn_sub(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_sub)
    }
    #[inline(always)]
    /// Make an instruction that subtracts the second value from the first and throws upon overflow
    pub fn insn_sub_ovf(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_sub_ovf)
    }
    #[inline(always)]
    /// Make an instruction that divides the first number by the second
    ///
    /// You can also just use `v1 / v2` in your code instead of running this method,
    /// `&Val` has the `Div` trait implemented so it can be divided with normal operators.
    pub fn insn_div(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_div)
    }
    #[inline(always)]
    /// Make an instruction that finds the remainder when the first number is
    /// divided by the second
    ///
    /// You can also just use `v1 % v2` in your code instead of running this method,
    /// `&Val` has the `Rem` trait implemented so it can be done with normal operators.
    pub fn insn_rem(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_rem)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is lower than or
    /// equal to the second
    pub fn insn_leq(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_le)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is greater than or
    /// equal to the second
    pub fn insn_geq(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_ge)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is lower than the second
    pub fn insn_lt(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_lt)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is greater than the second
    pub fn insn_gt(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_gt)
    }
    #[inline(always)]
    /// Make an instruction that checks if the values are equal
    pub fn insn_eq(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_eq)
    }
    #[inline(always)]
    /// Make an instruction that checks if the values are not equal
    pub fn insn_neq(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_ne)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise and on the two values
    ///
    /// You can also just use `v1 & v2` in your code instead of running this method,
    /// `&Val` has the `BitAnd` trait implemented so it can be done with normal operators.
    pub fn insn_and(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_and)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise or on the two values
    ///
    /// You can also just use `v1 | v2` in your code instead of running this method,
    /// `&Val` has the `BitOr` trait implemented so it can be done with normal operators.
    pub fn insn_or(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_or)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise xor on the two values
    ///
    /// You can also just use `v1 ^ v2` in your code instead of running this method,
    /// `&Val` has the `BitXor` trait implemented so it can be done with normal operators.
    pub fn insn_xor(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_xor)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise not on the two values
    ///
    /// You can also just use `!value` in your code instead of running this method.
    /// `&Val` has the `Not` trait implemented so it can be inversed with normal operators.
    pub fn insn_not(&self, value: &'a Val) -> &'a Val {
        self.insn_unop(value, jit_insn_not)
    }
    #[inline(always)]
    /// Make an instruction that performs a left bitwise shift on the first
    /// value by the second value
    ///
    /// You can also just use `v1 << v2` in your code instead of running this method,
    /// `&Val` has the `Shl` trait implemented so it can be shifted with normal operators.
    pub fn insn_shl(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_shl)
    }
    #[inline(always)]
    /// Make an instruction that performs a right bitwise shift on the first
    /// value by the second value
    ///
    /// You can also just use `v1 >> v2` in your code instead of running this method,
    /// `&Val` has the `Shr` trait implemented so it can be shifted with normal operators.
    pub fn insn_shr(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_shr)
    }
    /// Make an instruction that performs a right bitwise shift on the first
    /// value by the second value
    pub fn insn_ushr(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_ushr)
    }
    #[inline(always)]
    /// Make an instruction that performs a negation on the value
    ///
    /// You can also just use `-value` in your code instead of running this method.
    /// `&Val` has the `Neg` trait implemented so it can be negatedd with normal operators.
    pub fn insn_neg(&self, value: &'a Val) -> &'a Val {
        self.insn_unop(value, jit_insn_neg)
    }
    #[inline(always)]
    /// Make an instruction that duplicates the value given
    ///
    /// This is the same as load
    pub fn insn_dup(&self, value: &'a Val) -> &'a Val {
        unsafe {
            let dup_value = jit_insn_load(self.into(), value.into());
            from_ptr(dup_value)
        }
    }
    #[inline(always)]
    /// Make an instruction that loads the contents of `src` into a temporary
    pub fn insn_load(&self, src: &'a Val) -> &'a Val {
        self.insn_unop(src, jit_insn_load)
    }
    #[inline(always)]
    /// Make an instruction that loads a value of the given type from `value + offset`, where
    /// `value` must be a pointer
    pub fn insn_load_relative(&self, value: &'a Val, offset: usize, ty: &Ty) -> &'a Val {
        if cfg!(not(ndebug)) && !value.get_type().is_pointer() {
            panic!("Value given to insn_load_relative should be pointer, got {:?}", value.get_type());
        }
        unsafe {
            from_ptr(jit_insn_load_relative(
                self.into(),
                value.into(),
                offset as jit_nint,
                ty.into()
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that stores the contents of `val` into `dest`, where `dest` is a
    /// temporary value or local value
    pub fn insn_store(&self, dest: &'a Val, val: &'a Val) {
        unsafe {
            jit_insn_store(self.into(), dest.into(), val.into());
        }
    }
    #[inline(always)]
    /// Make an instruction that stores the `value` at the address `dest + offset`, where `dest`
    /// must be a pointer
    pub fn insn_store_relative(&self, dest: &'a Val, offset: usize, value: &'a Val) {
        if cfg!(not(ndebug)) && !dest.get_type().is_pointer() {
            panic!("Destination given to insn_store_relative should be pointer, got {:?}", value.get_type());
        }
        unsafe {
            jit_insn_store_relative(self.into(), dest.into(), offset as jit_nint, value.into());
        }
    }
    #[inline(always)]
    /// Make an instruction that sets a label
    pub fn insn_label(&self, label: &mut Label<'a>) {
        unsafe {
            jit_insn_label(self.into(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label
    pub fn insn_branch(&self, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch(self.into(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label if the value is true
    pub fn insn_branch_if(&self, value: &'a Val, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch_if(self.into(), value.into(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label if the value is false
    pub fn insn_branch_if_not(&self, value: &'a Val, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch_if_not(self.into(), value.into(), &mut **label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a label in the table
    pub fn insn_jump_table(&self, value: &'a Val, labels: &mut [Label<'a>]) {
        unsafe {
            let mut native_labels: Vec<_> = labels.iter()
                .map(|label| **label).collect();
            jit_insn_jump_table(
                self.into(),
                value.into(),
                native_labels.as_mut_ptr(),
                labels.len() as c_uint
            );
        }
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse cosine of the number given
    pub fn insn_acos(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_acos)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse sine of the number given
    pub fn insn_asin(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_asin)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse tangent of the number given
    pub fn insn_atan(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_atan)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse tangent of the numbers given
    pub fn insn_atan2(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_atan2)
    }
    #[inline(always)]
    /// Make an instruction that finds the nearest integer above a number
    pub fn insn_ceil(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_ceil)
    }
    #[inline(always)]
    /// Make an instruction that gets the consine of the number given
    pub fn insn_cos(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_cos)
    }
    #[inline(always)]
    /// Make an instruction that gets the hyperbolic consine of the number given
    pub fn insn_cosh(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_cosh)
    }
    #[inline(always)]
    /// Make an instruction that gets the natural logarithm rased to the power
    /// of the number
    pub fn insn_exp(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_exp)
    }
    #[inline(always)]
    /// Make an instruction that finds the nearest integer below a number
    pub fn insn_floor(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_floor)
    }
    #[inline(always)]
    /// Make an instruction that gets the natural logarithm of the number
    pub fn insn_log(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_log)
    }
    #[inline(always)]
    /// Make an instruction that gets the base 10 logarithm of the number
    pub fn insn_log10(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_log10)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of raising the first value to
    /// the power of the second value
    pub fn insn_pow(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        self.insn_binop(v1, v2, jit_insn_pow)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of rounding the value to the
    /// nearest integer
    pub fn insn_rint(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_rint)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of rounding the value to the
    /// nearest integer
    pub fn insn_round(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_round)
    }
    #[inline(always)]
    /// Make an instruction the gets the sine of the number
    pub fn insn_sin(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_sin)
    }
    #[inline(always)]
    /// Make an instruction the gets the hyperbolic sine of the number
    pub fn insn_sinh(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_sinh)
    }
    #[inline(always)]
    /// Make an instruction the gets the square root of a number
    pub fn insn_sqrt(&self, value: &'a Val) -> &'a Val {
        expect!(insn_sqrt, value, float);
        self.insn_unop(value, jit_insn_sqrt)
    }
    #[inline(always)]
    /// Make an instruction the gets the tangent of a number
    pub fn insn_tan(&self, v: &'a Val) -> &'a Val {
        self.insn_unop(v, jit_insn_tan)
    }
    #[inline(always)]
    /// Make an instruction the gets the hyperbolic tangent of a number
    pub fn insn_tanh(&self, v: &'a Val) -> &'a Val{
        self.insn_unop(v, jit_insn_tanh)
    }
    #[inline(always)]
    /// Make an instruction that truncates the value
    pub fn insn_trunc(&self, v: &'a Val) -> &'a Val {
        self.insn_unop(v, jit_insn_trunc)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is NaN
    pub fn insn_is_nan(&self, v: &'a Val) -> &'a Val {
        expect!(insn_is_nan, v, float);
        self.insn_unop(v, jit_insn_is_nan)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is finite
    pub fn insn_is_finite(&self, v: &'a Val) -> &'a Val {
        expect!(insn_is_finite, v, float);
        self.insn_unop(v, jit_insn_is_finite)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is  infinite
    pub fn insn_is_inf(&self, v: &'a Val) -> &'a Val {
        expect!(insn_is_inf, v, float);
        self.insn_unop(v, jit_insn_is_inf)
    }
    #[inline(always)]
    /// Make an instruction that gets the absolute value of a number
    pub fn insn_abs(&self, v: &'a Val) -> &'a Val {
        expect!(insn_abs, v, primitive);
        self.insn_unop(v, jit_insn_abs)
    }
    #[inline(always)]
    /// Make an instruction that gets the smallest of two numbers
    pub fn insn_min(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        expect!(insn_min, v1, v2, primitive);
        self.insn_binop(v1, v2, jit_insn_min)
    }
    #[inline(always)]
    /// Make an instruction that gets the biggest of two numbers
    pub fn insn_max(&self, v1: &'a Val, v2: &'a Val) -> &'a Val {
        expect!(insn_max, v1, v2, primitive);
        self.insn_binop(v1, v2, jit_insn_max)
    }
    #[inline(always)]
    /// Make an instruction that gets the sign of a number
    pub fn insn_sign(&self, v: &'a Val) -> &'a Val {
        expect!(insn_sign, v, primitive);
        self.insn_unop(v, jit_insn_sign)
    }

    /// Call the function, which may or may not be translated yet
    pub fn insn_call(&self, name:Option<&str>, func:&Func, sig:Option<&Ty>,
        args: &mut [&'a Val], flags: flags::CallFlags) -> &'a Val {
        unsafe {
            let native_args:&mut [jit_value_t] = mem::transmute(args);
            let c_name = name.map(|name| CString::new(name.as_bytes()).unwrap());
            let sig = mem::transmute(sig);
            from_ptr(jit_insn_call(
                self.into(),
                c_name.map(|name| name.as_bytes().as_ptr() as *mut c_char).unwrap_or(ptr::null_mut()),
                func.into(), sig, native_args.as_mut_ptr(),
                native_args.len() as c_uint,
                flags.bits()
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that calls a function that has the signature given
    /// with some arguments through a pointer to the fucntion
    pub fn insn_call_indirect(&self, func:&'a Val, signature: &Ty,
                               args: &mut [&'a Val], flags: flags::CallFlags) -> &'a Val {
        unsafe {
            let native_args: &mut [jit_value_t] = mem::transmute(args);
            from_ptr(jit_insn_call_indirect(
                self.into(),
                func.into(),
                signature.into(),
                native_args.as_mut_ptr(),
                native_args.len() as c_uint,
                flags.bits()
            ))
        }
    }
    /// Make an instruction that calls a native function that has the signature
    /// given with some arguments
    fn insn_call_native(&self, name: Option<&str>,
                        native_func: *mut c_void, signature: &Ty,
                        args: &mut [&'a Val], flags: flags::CallFlags) -> &'a Val {
        if cfg!(not(ndebug)) {
            let name = name.unwrap_or("unnamed function");
            if !signature.is_signature() {
                panic!("Bad signature for {} - expected signature, got {:?}", name, signature)
            }
            let num_sig_args = signature.params().count();
            if args.len() != num_sig_args {
                panic!("Bad arguments to {} - expected {}, got {}", name, num_sig_args, args.len());
            }
            for (index, (arg, param)) in args.iter().zip(signature.params()).enumerate() {
                let ty = arg.get_type();
                if ty != param {
                    panic!("Bad argument #{} to {} - expected {:?}, got {:?}", index, name, param, ty);
                }
            }
        }
        unsafe {
            let native_args:&mut [jit_value_t] = mem::transmute(args);
            let c_name = name.map(|name| CString::new(name.as_bytes()).unwrap());
            from_ptr(jit_insn_call_native(
                self.into(),
                c_name.map(|name| name.as_bytes().as_ptr() as *mut c_char).unwrap_or(ptr::null_mut()),
                native_func,
                signature.into(),
                native_args.as_mut_ptr(),
                native_args.len() as c_uint,
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
                            flags: flags::CallFlags) -> &'a Val {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, &mut [], flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with a single argument and expects a return value
    pub fn insn_call_native1<A,R>(&self, name: Option<&str>,
                                native_func: extern fn(A) -> R,
                                signature: &Ty,
                                mut args: [&'a Val; 1],
                                flags: flags::CallFlags) -> &'a Val {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, &mut args, flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with two arguments and expects a return value
    pub fn insn_call_native2<A,B,R>(&self, name: Option<&str>,
                                native_func: extern fn(A, B) -> R,
                                signature: &Ty,
                                mut args: [&'a Val; 2],
                                flags: flags::CallFlags) -> &'a Val {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, &mut args, flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with three arguments and expects a return value
    pub fn insn_call_native3<A,B,C,R>(&self, name: Option<&str>,
                                native_func: extern fn(A, B, C) -> R,
                                signature: &Ty,
                                mut args: [&'a Val; 3],
                                flags: flags::CallFlags) -> &'a Val {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, &mut args, flags)
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with four arguments and expects a return value
    pub fn insn_call_native4<A,B,C,D,R>(&self, name: Option<&str>,
                                native_func: extern fn(A, B, C, D) -> R,
                                signature: &Ty,
                                mut args: [&'a Val; 4],
                                flags: flags::CallFlags) -> &'a Val {
        let func_ptr = unsafe { mem::transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, &mut args
            , flags)
    }
    #[inline(always)]
    /// Make an instruction that copies `size` bytes from the `source` address to the `dest` address
    pub fn insn_memcpy(&self, dest: &'a Val, source: &'a Val, size: &'a Val) -> bool {
        expect!(insn_memcpy, dest, source, size);
        unsafe {
            jit_insn_memcpy(self.into(), dest.into(), source.into(), size.into()) != 0
        }
    }
    #[inline(always)]
    /// Make an instruction that moves memory from a source address to a destination address
    pub fn insn_memmove(&self, dest: &'a Val, source: &'a Val, size: &'a Val) -> bool {
        expect!(insn_memmove, dest, source, size);
        unsafe {
            jit_insn_memmove(self.into(), dest.into(), source.into(), size.into()) != 0
        }
    }
    #[inline(always)]
    /// Make an instruction that sets memory at the destination address
    pub fn insn_memset(&self, dest: &'a Val, source: &'a Val, size: &'a Val) -> bool {
        expect!(insn_memset, dest, source, size);
        unsafe {
            jit_insn_memset(self.into(), dest.into(), source.into(), size.into()) != 0
        }
    }
    #[inline(always)]
    /// Make an instruction that allocates `size` bytes of memory from the stack
    pub fn insn_alloca(&self, size: &'a Val) -> &'a Val {
        expect!(insn_alloca, size, int);
        unsafe {
            from_ptr(jit_insn_alloca(self.into(), size.into()))
        }
    }
    #[inline(always)]
    /// Make an instruction that gets the address of a value
    pub fn insn_address_of(&self, value: &'a Val) -> &'a Val {
        unsafe {
            from_ptr(jit_insn_address_of(self.into(), value.into()))
        }
    }
    #[inline(always)]
    fn insn_binop(&self,
                    v1: &'a Val, v2: &'a Val,
                    f: unsafe extern "C" fn(
                        jit_function_t,
                        jit_value_t,
                        jit_value_t) -> jit_value_t)
                    -> &'a Val {
        unsafe {
            from_ptr(f(self.into(), v1.into(), v2.into()))
        }
    }
    #[inline(always)]
    fn insn_unop(&self,
                    value: &'a Val,
                    f: unsafe extern "C" fn(
                        jit_function_t,
                        jit_value_t) -> jit_value_t)
                    -> &'a Val {
        unsafe {
            from_ptr(f(self.into(), value.into()))
        }
    }
    #[inline(always)]
    /// Make instructions to run the block if the condition is met
    pub fn insn_if<B>(&self, cond: &'a Val, block: B) where B:FnOnce() {
        let mut after = Label::new(self);
        self.insn_branch_if_not(cond, &mut after);
        block();
        self.insn_label(&mut after);
    }
    #[inline(always)]
    /// Make instructions to run the block if the condition is not met
    pub fn insn_if_not<B>(&self, cond: &'a Val, block: B) where B:FnOnce() {
        let mut after = Label::new(self);
        self.insn_branch_if(cond, &mut after);
        block();
        self.insn_label(&mut after);
    }
    #[inline(always)]
    /// Make instructions to run the block if the condition is met
    pub fn insn_if_else<A, B>(&self, cond: &'a Val, if_block: A, else_block: B) where A:FnOnce(), B:FnOnce() {
        let mut after = Label::new(self);
        let mut end = Label::new(self);
        self.insn_branch_if_not(cond, &mut after);
        if_block();
        self.insn_branch(&mut end);
        self.insn_label(&mut after);
        else_block();
        self.insn_label(&mut end)
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
    pub fn insn_while<C, B>(&self, cond: C, block: B)
        where C:FnOnce() -> &'a Val, B:FnOnce() {
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
            jit_function_set_optimization_level(self.into(), level);
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
            jit_function_set_recompilable(self.into());
        }
    }
    /// Get the entry block of this function
    pub fn get_entry(&self) -> Option<Block<'a>> {
        unsafe {
            from_ptr_opt(jit_function_get_entry(self.into()))
        }
    }
    /// Get the current block of this function
    pub fn get_current(&self) -> Option<Block<'a>> {
        unsafe {
            from_ptr_opt(jit_function_get_current(self.into()))
        }
    }
    #[inline(always)]
    /// Compile the function
    pub fn compile(self) -> CompiledFunction<'a> {
        if !self.owned {
            panic!("The function must be owned")
        }
        unsafe {
            let ptr = (&self).into();
            mem::forget(self);
            jit_function_compile(ptr);
            from_ptr(ptr)
        }
    }
    #[inline(always)]
    /// Compile the function and call a closure with it directly
    pub fn compile_with<A, R, F>(self, cb: F) -> CompiledFunction<'a>
        where F:FnOnce(extern fn(A) -> R) {
        let compiled = self.compile();
        compiled.with(cb);
        compiled
    }
}
