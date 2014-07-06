use bindings::*;
use context::{
    Context,
    InContext
};
use compile::Compile;
use label::Label;
use types::Type;
use util::NativeRef;
use value::Value;
use libc::{
    c_int,
    c_uint,
    c_void
};
use std::kinds::marker::ContravariantLifetime;
use std::mem::transmute;
use std::ptr::mut_null;
use std::c_str::ToCStr;
/// A platform's application binary interface
pub enum ABI {
    /// The C application binary interface
    CDECL = 0
}
/// Call flags to a function
pub enum CallFlags {
    /// When the function won't throw a value
    JitCallNothrow = 1,
    /// When the function won't return a value
    JitCallNoReturn = 2,
    /// When the function is tail-recursive
    JitCallTail = 4,
}
#[deriving(PartialEq)]
/// A function persists for the lifetime of its containing context. It initially
/// starts life in the "building" state, where the user constructs instructions
/// that represents the function body. Once the build process is complete, the
/// user calls `function.compile()` to convert it into its executable form.
pub struct Function<'a> {
    _func: jit_function_t,
    marker: ContravariantLifetime<'a>
}
impl<'a> NativeRef for Function<'a> {
    #[inline(always)]
    /// Convert to a native pointer
    unsafe fn as_ptr(&self) -> jit_function_t {
        self._func
    }
    #[inline(always)]
    /// Convert from a native pointer
    unsafe fn from_ptr(ptr:jit_function_t) -> Function<'a> {
        Function {
            _func: ptr,
            marker: ContravariantLifetime::<'a>
        }
    }
}
#[unsafe_destructor]
impl<'a> Drop for Function<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            jit_function_abandon(self.as_ptr());
        }
    }
}
impl<'a> InContext<'a> for Function<'a> {
    #[inline(always())]
    /// Get the context this function was made in
    fn get_context(&self) -> Context<'a> {
        unsafe {
            let context = jit_function_get_context(self.as_ptr());
            NativeRef::from_ptr(context)
        }
    }
}
impl<'a> Function<'a> {
    #[inline(always)]
    /// Create a new function block and associate it with a JIT context.
    /// It is recommended that you call `Function::new` and `function.compile()`
    /// in the closure you give to `context.build(...)`.
    /// 
    /// This will protect the JIT's internal data structures within a
    /// multi-threaded environment.
    pub fn new(context:&'a Context<'a>,
               signature:Type) -> Function<'a> {
        unsafe {
            NativeRef::from_ptr(jit_function_create(
                context.as_ptr(),
                signature.as_ptr()
            ))
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
    pub fn new_nested(context:&'a Context<'a>,
                        signature: Type,
                        parent: &'a Function<'a>) -> Function<'a> {
        unsafe {
            NativeRef::from_ptr(jit_function_create_nested(
                context.as_ptr(),
                signature.as_ptr(),
                parent.as_ptr()
            ))
        }
    }
    #[inline(always)]
    fn insn_binop(&self,
                    v1: &Value<'a>, v2: &Value<'a>,
                    f: unsafe extern "C" fn(
                        jit_function_t,
                        jit_value_t,
                        jit_value_t) -> jit_value_t)
                    -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(f(self.as_ptr(), v1.as_ptr(), v2.as_ptr()))
        }
    }
    #[inline(always)]
    fn insn_unop(&self,
                    value: &Value<'a>,
                    f: unsafe extern "C" fn(
                        jit_function_t,
                        jit_value_t) -> jit_value_t)
                    -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(f(self.as_ptr(), value.as_ptr()))
        }
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
    /// Make this function a candidate for recompilation
    pub fn set_recompilable(&self) {
        unsafe {
            jit_function_set_recompilable(self.as_ptr());
        }
    }
    #[inline(always)]
    /// Compile the function
    pub fn compile(&self) {
        unsafe {
            jit_function_compile(self.as_ptr());
        }
    }
    /// Get the value that corresponds to a specified function parameter.
    pub fn get_param(&self, param: uint) -> Value<'a> {
        unsafe {
            let value = jit_value_get_param(self.as_ptr(), param as c_uint);
            NativeRef::from_ptr(value)
        }
    }
    #[inline(always)]
    /// Make an instructional representation of a Rust value
    pub fn insn_of<T:Compile>(&'a self, val:&T) -> Value<'a> {
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
    pub fn insn_throw(&self, retval: &Value<'a>) {
        unsafe {
            jit_insn_throw(self.as_ptr(), retval.as_ptr());
        }
    }
    #[inline(always)]
    /// Return from the function with the value given
    pub fn insn_return(&self, retval: &Value<'a>) {
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
    pub fn insn_mul(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_mul)
    }
    #[inline(always)]
    /// Make an instruction that adds the values
    pub fn insn_add(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_add)
    }
    #[inline(always)]
    /// Make an instruction that subtracts the second value from the first
    pub fn insn_sub(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_sub)
    }
    #[inline(always)]
    /// Make an instruction that divides the first number by the second
    pub fn insn_div(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_div)
    }
    #[inline(always)]
    /// Make an instruction that finds the remainder when the first number is
    /// divided by the second
    pub fn insn_rem(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_rem)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is lower than or
    /// equal to the second
    pub fn insn_leq(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_le)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is greater than or
    /// equal to the second
    pub fn insn_geq(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_ge)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is lower than the second
    pub fn insn_lt(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_lt)
    }
    #[inline(always)]
    /// Make an instruction that checks if the first value is greater than the second
    pub fn insn_gt(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_gt)
    }
    #[inline(always)]
    /// Make an instruction that checks if the values are equal
    pub fn insn_eq(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_eq)
    }
    #[inline(always)]
    /// Make an instruction that checks if the values are not equal
    pub fn insn_neq(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_ne)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise and on the two values
    pub fn insn_and(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_and)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise or on the two values
    pub fn insn_or(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_or)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise xor on the two values
    pub fn insn_xor(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_xor)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise not on the two values
    pub fn insn_not(&self, value: &Value<'a>) -> Value<'a> {
        self.insn_unop(value, jit_insn_not)
    }
    #[inline(always)]
    /// Make an instruction that performs a left bitwise shift on the first
    /// value by the second value
    pub fn insn_shl(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_shl)
    }
    #[inline(always)]
    /// Make an instruction that performs a right bitwise shift on the first
    /// value by the second value
    pub fn insn_shr(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_shr)
    }
    /// Make an instruction that performs a right bitwise shift on the first
    /// value by the second value
    pub fn insn_ushr(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_ushr)
    }
    #[inline(always)]
    /// Make an instruction that performs a bitwise negate on the value
    pub fn insn_neg(&self, value: &Value<'a>) -> Value<'a> {
        self.insn_unop(value, jit_insn_neg)
    }
    #[inline(always)]
    /// Make an instruction that duplicates the value given
    pub fn insn_dup(&self, value: &Value<'a>) -> Value<'a> {
        unsafe {
            let dup_value = jit_insn_load(self.as_ptr(), value.as_ptr());
            NativeRef::from_ptr(dup_value)
        }
    }
    #[inline(always)]
    /// Make an instruction that loads a value from a src value
    pub fn insn_load(&self, src: &Value<'a>) -> Value<'a> {
        self.insn_unop(src, jit_insn_load)
    }
    #[inline(always)]
    /// Make an instruction that loads a value from a src value
    pub fn insn_load_relative(&self, src: &Value<'a>, offset: int, ty:Type) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_insn_load_relative(
                self.as_ptr(),
                src.as_ptr(),
                offset as jit_nint,
                ty.as_ptr()
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that stores a value at a destination value
    pub fn insn_store(&self, dest: &Value<'a>, src: &Value<'a>) {
        unsafe {
            jit_insn_store(self.as_ptr(), dest.as_ptr(), src.as_ptr());
        }
    }
    #[inline(always)]
    /// Make an instruction that stores a value a certain offset away from a
    /// destination value
    pub fn insn_store_relative(&self, dest: &Value<'a>, offset: int, 
                               src: &Value<'a>) {
        unsafe {
            jit_insn_store_relative(self.as_ptr(), dest.as_ptr(), offset as jit_nint, src.as_ptr());
        }
    }
    #[inline(always)]
    /// Make an instruction that sets a label
    pub fn insn_set_label(&self, label: &mut Label<'a>) {
        unsafe {
            jit_insn_label(self.as_ptr(), &mut (label.get_value() as jit_label_t));
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label
    pub fn insn_branch(&self, label: &mut Label<'a>) {
        unsafe {
            jit_insn_branch(self.as_ptr(), &mut (label.get_value() as jit_label_t));
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label if the value is true
    pub fn insn_branch_if(&self, value: &Value<'a>, label: &mut Label<'a>) {
        unsafe {
            let mut native_label = label.get_value() as jit_label_t;
            jit_insn_branch_if(self.as_ptr(), value.as_ptr(), &mut native_label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a certain label if the value is false
    pub fn insn_branch_if_not(&self, value: &Value<'a>, label: &mut Label<'a>) {
        unsafe {
            let mut native_label = label.get_value() as jit_label_t;
            jit_insn_branch_if_not(self.as_ptr(), value.as_ptr(), &mut native_label);
        }
    }
    #[inline(always)]
    /// Make an instruction that branches to a label in the table
    pub fn insn_jump_table(&self, value: &Value<'a>, labels: &mut [Label<'a>]) {
        unsafe {
            let mut native_labels: Vec<_> = labels.iter()
                .map(|label|label.get_value() as jit_label_t).collect();
            jit_insn_jump_table(
                self.as_ptr(),
                value.as_ptr(),
                native_labels.as_mut_ptr(),
                labels.len() as c_uint
            );
        }
    }
    /// Call the function, which may or may not be translated yet
    pub fn insn_call<S:ToCStr>(&self, name:Option<S>, func:&Function<'a>,
                                sig:Option<Type>,
                                args: &mut [&Value<'a>]) -> Value<'a> {
        unsafe {
            let mut native_args:Vec<jit_value_t> = args.iter().map(|arg| arg.as_ptr()).collect();
            let cb = |c_name|
                NativeRef::from_ptr(jit_insn_call(self.as_ptr(), c_name, func.as_ptr(), sig.as_ptr(), native_args.as_mut_ptr(), args.len() as c_uint, JitCallNothrow as c_int));
            match name {
                Some(ref name) => name.with_c_str(cb),
                None => cb(RawPtr::null())
            }
        }
    }
    #[inline(always)]
    /// Make an instruction that calls a function that has the signature given
    /// with some arguments
    pub fn insn_call_indirect(&self, func:&Function<'a>, signature: Type,
                               args: &mut [&Value<'a>]) -> Value<'a> {
        unsafe {
            let mut native_args:Vec<jit_value_t> = args.iter().map(|arg| arg.as_ptr()).collect();
            NativeRef::from_ptr(jit_insn_call_indirect(self.as_ptr(), func.as_ptr(), signature.as_ptr(), native_args.as_mut_ptr(), args.len() as c_uint, JitCallNothrow as c_int))
        }
    }
    /// Make an instruction that calls a native function that has the signature
    /// given with some arguments
    fn insn_call_native<S:ToCStr>(&self, name: Option<S>,
                        native_func: *mut c_void, signature: Type,
                        args: &mut [&Value<'a>]) -> Value<'a> {
        unsafe {
            let mut native_args:Vec<jit_value_t> = args.iter()
                .map(|arg| arg.as_ptr()).collect();
            let cb = |c_name| {
                NativeRef::from_ptr(jit_insn_call_native(
                    self.as_ptr(),
                    c_name,
                    native_func,
                    signature.as_ptr(),
                    native_args.as_mut_ptr(),
                    args.len() as c_uint,
                    JitCallNothrow as c_int
                ))
            };
            match name {
                Some(ref name) => name.with_c_str(cb),
                None => cb(RawPtr::null())
            }
        }
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with no arguments and expects a return value
    pub fn insn_call_native0<R, S:ToCStr>(&self, name: Option<S>,
                                native_func: fn() -> R,
                                signature: Type,
                                mut args: [&Value<'a>, ..0]) -> Value<'a> {
        let func_ptr = unsafe { transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice())
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with a single argument and expects a return value
    pub fn insn_call_native1<A,R, S:ToCStr>(&self, name: Option<S>,
                                  native_func: fn(A) -> R,
                                  signature: Type,
                                  mut args: [&Value<'a>, ..1]) -> Value<'a> {
        let func_ptr = unsafe { transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice())
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with two arguments and expects a return value
    pub fn insn_call_native2<A,B,R, S:ToCStr>(&self, name: Option<S>,
                                  native_func: fn(A, B) -> R,
                                  signature: Type,
                                  mut args: [&Value<'a>, ..2]) -> Value<'a> {
        let func_ptr = unsafe { transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice())
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with three arguments and expects a return value
    pub fn insn_call_native3<A,B,C,R, S:ToCStr>(&self, name: Option<S>,
                                  native_func: fn(A, B, C) -> R,
                                  signature: Type,
                                  mut args: [&Value<'a>, ..3]) -> Value<'a> {
        let func_ptr = unsafe { transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice())
    }
    #[inline(always)]
    /// Make an instruction that calls a Rust function that has the signature
    /// given with four arguments and expects a return value
    pub fn insn_call_native4<A,B,C,D,R, S:ToCStr>(&self, name: Option<S>,
                                  native_func: fn(A, B, C, D) -> R,
                                  signature: Type,
                                  mut args: [&Value<'a>, ..4]) -> Value<'a> {
        let func_ptr = unsafe { transmute(native_func) };
        self.insn_call_native(name, func_ptr, signature, args.as_mut_slice())
    }
    #[inline(always)]
    /// Make an instruction that allocates some space
    pub fn insn_alloca(&self, size: &Value<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_insn_alloca(self.as_ptr(), size.as_ptr()))
        }
    }
    #[inline(always)]
    /// Apply a function to some arguments and set the retval to the return value
    pub fn apply<T>(&self, args: &mut [*mut c_void], retval: &mut T) {
        unsafe {
            jit_function_apply(self.as_ptr(), args.as_mut_ptr(), transmute(retval));
        }
    }
    #[inline(always)]
    /// Execute a function and with some arguments
    pub fn execute(&self, args: &mut [*mut c_void]) {
        unsafe {
            jit_function_apply(self.as_ptr(), args.as_mut_ptr(), mut_null());
        }
    }
    #[inline(always)]
    /// Turn this function into a closure
    pub unsafe fn closure(&self) -> *mut c_void {
        jit_function_to_closure(self.as_ptr())
    }
    #[inline(always)]
    /// Turn this function into a closure with 0 arguments
    pub fn with_closure0<Y, Z>(&self, cb:|fn() -> Z| -> Y) -> Y {
        unsafe {
            cb(transmute(self.closure()))
        }
    }
    #[inline(always)]
    /// Turn this function into a closure with 1 argument
    pub fn with_closure1<A, Y, Z>(&self, cb:|fn(A) -> Z| -> Y)  -> Y {
        unsafe {
            cb(transmute(self.closure()))
        }
    }
    #[inline(always)]
    /// Turn this function into a closure with 2 arguments
    pub fn with_closure2<A, B, Y, Z>(&self, cb:|fn(A, B) -> Z| -> Y) -> Y {
        unsafe {
            cb(transmute(self.closure()))
        }
    }
    #[inline(always)]
    /// Turn this function into a closure with 3 arguments
    pub fn with_closure3<A, B, C, Y, Z>(&self, cb:|fn(A, B, C) -> Z| -> Y) -> Y {
        unsafe {
            cb(transmute(self.closure()))
        }
    }
    #[inline(always)]
    /// Make an instruction that converts the value to the type given
    pub fn insn_convert(&self, v: &Value<'a>,
                            t:Type, overflow_check:bool) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_insn_convert(
                self.as_ptr(),
                v.as_ptr(),
                t.as_ptr(),
                overflow_check as c_int
            ))
        }
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse cosine of the number given
    pub fn insn_acos(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_acos)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse sine of the number given
    pub fn insn_asin(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_asin)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse tangent of the number given
    pub fn insn_atan(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_atan)
    }
    #[inline(always)]
    /// Make an instruction that gets the inverse tangent of the numbers given
    pub fn insn_atan2(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_atan2)
    }
    #[inline(always)]
    /// Make an instruction that finds the nearest integer above a number
    pub fn insn_ceil(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_ceil)
    }
    #[inline(always)]
    /// Make an instruction that gets the consine of the number given
    pub fn insn_cos(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_cos)
    }
    #[inline(always)]
    /// Make an instruction that gets the hyperbolic consine of the number given
    pub fn insn_cosh(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_cosh)
    }
    #[inline(always)]
    /// Make an instruction that gets the natural logarithm rased to the power
    /// of the number
    pub fn insn_exp(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_exp)
    }
    #[inline(always)]
    /// Make an instruction that finds the nearest integer below a number
    pub fn insn_floor(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_floor)
    }
    #[inline(always)]
    /// Make an instruction that gets the natural logarithm of the number
    pub fn insn_log(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_log)
    }
    #[inline(always)]
    /// Make an instruction that gets the base 10 logarithm of the number
    pub fn insn_log10(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_log10)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of raising the first value to
    /// the power of the second value
    pub fn insn_pow(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_pow)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of rounding the value to the
    /// nearest integer
    pub fn insn_rint(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_rint)
    }
    #[inline(always)]
    /// Make an instruction the gets the result of rounding the value to the
    /// nearest integer
    pub fn insn_round(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_round)
    }
    #[inline(always)]
    /// Make an instruction the gets the sine of the number
    pub fn insn_sin(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sin)
    }
    #[inline(always)]
    /// Make an instruction the gets the hyperbolic sine of the number
    pub fn insn_sinh(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sinh)
    }
    #[inline(always)]
    /// Make an instruction the gets the square root of a number
    pub fn insn_sqrt(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sqrt)
    }
    #[inline(always)]
    /// Make an instruction the gets the tangent of a number
    pub fn insn_tan(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_tan)
    }
    #[inline(always)]
    /// Make an instruction the gets the hyperbolic tangent of a number
    pub fn insn_tanh(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_tanh)
    }
    #[inline(always)]
    /// Make an instruction that truncates the value
    pub fn insn_trunc(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_trunc)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is NaN
    pub fn insn_is_nan(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_is_nan)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is finite
    pub fn insn_is_finite(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_is_finite)
    }
    #[inline(always)]
    /// Make an instruction that checks if the number is  infinite
    pub fn insn_is_inf(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_is_inf)
    }
    #[inline(always)]
    /// Make an instruction that gets the absolute value of a number
    pub fn insn_abs(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_abs)
    }
    #[inline(always)]
    /// Make an instruction that gets the smallest of two numbers
    pub fn insn_min(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_min)
    }
    #[inline(always)]
    /// Make an instruction that gets the biggest of two numbers
    pub fn insn_max(&self, v1: &Value<'a>, v2: &Value<'a>) -> Value<'a> {
        self.insn_binop(v1, v2, jit_insn_max)
    }
    #[inline(always)]
    /// Make an instruction that gets the sign of a number
    pub fn insn_sign(&self, v: &Value<'a>) -> Value<'a>{
        self.insn_unop(v, jit_insn_sign)
    }
}