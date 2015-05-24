use raw::*;
use function::UncompiledFunction;
use types::*;
use util::from_ptr;
use std::marker::PhantomData;
use std::fmt;
use std::ops::*;
/// Vals form the backbone of the storage system in `LibJIT`
///
/// Every value in the system, be it a constant, a local variable, or a
/// temporary result, is represented by an object of type `Val`. The JIT then
/// allocates registers or memory locations to the values as appropriate. This is
/// why `Val` is always behind a reference
pub struct Val(PhantomData<[()]>);
native_ref!(&Val = LLVMValueRef);
impl fmt::Debug for Val {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "v({:?})", self.get_type())
    }
}
impl Val {
    #[inline(always)]
    /// Create a new value in the context of a function's current block.
    ///
    /// The value initially starts off as a block-specific temporary. It will be
    /// converted into a function-wide local variable if it is ever referenced
    /// from a different block.
    pub fn new<'a>(func:&UncompiledFunction<'a>, value_type:&Ty) -> &'a Val {
        unsafe {
            from_ptr(jit_value_create(func.into(), value_type.into()))
        }
    }
    /// Get the type of the value
    pub fn get_type(&self) -> &Ty {
        unsafe {
            from_ptr(jit_value_get_type(self.into()))
        }
    }
    /// Get the function which made this value
    pub fn get_function(&self) -> UncompiledFunction {
        unsafe {
            from_ptr(jit_value_get_function(self.into()))
        }
    }
    /// Determine if a value is temporary.  i.e. its scope extends over a single
    /// block within its function.
    #[inline]
    pub fn is_temp(&self) -> bool {
        unsafe {
            jit_value_is_temporary(self.into()) != 0
        }
    }
    /// Determine if a value is addressable.
    #[inline]
    pub fn is_addressable(&self) -> bool {
        unsafe {
            jit_value_is_addressable(self.into()) != 0
        }
    }
    /// Set a flag on a value to indicate that it is addressable.
    /// This should be used when you want to take the address of a value (e.g.
    /// `&variable` in Rust/C).  The value is guaranteed to not be stored in a
    /// register across a function call.
    #[inline]
    pub fn set_addressable(&self) -> () {
        unsafe {
            jit_value_set_addressable(self.into())
        }
    }
}
macro_rules! bin_op {
    ($trait_ty:ident, $trait_func:ident, $func:ident) => (
        impl<'a> $trait_ty<&'a Val> for &'a Val {
            type Output = &'a Val;
            fn $trait_func(self, other: &'a Val) -> &'a Val {
                self.get_function().$func(self, other)
            }
        }
    )
}
macro_rules! un_op {
    ($trait_ty:ident, $trait_func:ident, $func:ident) => (
        impl<'a> $trait_ty for &'a Val {
            type Output = &'a Val;
            fn $trait_func(self) -> &'a Val {
                self.get_function().$func(self)
            }
        }
    )
}
bin_op!{Add, add, insn_add}
bin_op!{BitAnd, bitand, insn_and}
bin_op!{BitOr, bitor, insn_or}
bin_op!{BitXor, bitxor, insn_xor}
bin_op!{Div, div, insn_div}
bin_op!{Mul, mul, insn_mul}
bin_op!{Rem, rem, insn_rem}
bin_op!{Shl, shl, insn_shl}
bin_op!{Shr, shr, insn_shr}
bin_op!{Sub, sub, insn_sub}
un_op!{Neg, neg, insn_neg}
un_op!{Not, not, insn_not}
