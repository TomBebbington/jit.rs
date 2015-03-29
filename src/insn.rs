use raw::*;
use function::Func;
use types::Ty;
use util::{from_ptr, from_ptr_opt};
use value::Value;
use std::{ffi, fmt, mem, str};
use std::marker::PhantomData;

/// Represents a single LibJIT instruction
pub struct Instruction<'a> {
    _insn: jit_insn_t,
    marker: PhantomData<&'a ()>
}
native_ref!(contra Instruction, _insn: jit_insn_t);
impl<'a> Copy for Instruction<'a> {}

impl<'a> Instruction<'a> {
	/// Get the opcode of the instruction
	pub fn get_opcode(self) -> i32 {
		unsafe {
			jit_insn_get_opcode(self._insn)
		}
	}
	/// Get the destination value
	pub fn get_dest(self) -> Option<Value<'a>> {
		unsafe {
			from_ptr_opt(jit_insn_get_dest(self._insn))
		}
	}
	/// Get if the destination value is a value
	pub fn dest_is_value(self) -> bool {
		unsafe {
			jit_insn_dest_is_value(self._insn) != 0
		}
	}
	/// Get the left value
	pub fn get_value1(self) -> Option<Value<'a>> {
		unsafe {
			from_ptr_opt(jit_insn_get_value1(self._insn))
		}
	}
	/// Get the right value
	pub fn get_value2(self) -> Option<Value<'a>> {
		unsafe {
			from_ptr_opt(jit_insn_get_value2(self._insn))
		}
	}
	/// Get the function containing this value
	pub fn get_function(self) -> Option<&'a Func> {
		unsafe {
			from_ptr_opt(jit_insn_get_function(self._insn))
		}
	}
	/// Get the signature of this value
	pub fn get_signature(self) -> Option<&'a Ty> {
		unsafe {
			from_ptr_opt(jit_insn_get_signature(self._insn))
		}
	}
	/// Get the name of the instruction
	pub fn get_name(self) -> &'a str {
		unsafe {
			let c_name = jit_insn_get_name(self._insn);
            let c_name = ffi::CStr::from_ptr(c_name);
			str::from_utf8(c_name.to_bytes()).unwrap()
		}
	}
}
impl<'a> fmt::Display for Instruction<'a> {
	fn fmt(&self, fmt:&mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.get_name())
	}
}

pub struct InstructionIter<'a> {
	_iter: jit_insn_iter_t,
	marker: PhantomData<&'a ()>,
}
impl<'a> Iterator for InstructionIter<'a> {
	type Item = Instruction<'a>;
	fn next(&mut self) -> Option<Instruction<'a>> {
		unsafe {
			let ptr = jit_insn_iter_next(&mut self._iter);
            if ptr.is_null() {
                None
            } else {
                Some(from_ptr(ptr))
            }
		}
	}
}

/// Represents a single LibJIT block
pub struct Block<'a> {
    _block: jit_block_t,
    marker: PhantomData<&'a ()>
}
native_ref!(contra Block, _block: jit_block_t);
impl<'a> Copy for Block<'a> {}
impl<'a> Block<'a> {
	/// Get the function containing this block
	pub fn get_function(self) -> &'a Func {
		unsafe {
			from_ptr(jit_block_get_function(self._block))
		}
	}
	/// Check if the block is reachable
	pub fn is_reachable(self) -> bool {
		unsafe {
			jit_block_is_reachable(self._block) != 0
		}
	}
	/// Check if the block ends in dead code
	pub fn ends_in_dead(self) -> bool {
		unsafe {
			jit_block_ends_in_dead(self._block) != 0
		}
	}
	/// Iterate through the instructions
	pub fn iter(self) -> InstructionIter<'a> {
		unsafe {
			let mut iter = mem::zeroed();
			jit_insn_iter_init(&mut iter, self._block);
			assert!(iter.block == self._block);
			assert!(iter.posn == 0);
			InstructionIter {
				_iter: iter,
				marker: PhantomData
			}
		}
	}
}
