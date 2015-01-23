use raw::*;
use std::marker::ContravariantLifetime;
use std::{ffi, fmt, mem, str};
use function::AnyFunction;
use value::Value;
use types::Type;
use util::{from_ptr, NativeRef};

/// Represents a single LibJIT instruction
native_ref!(Instruction ContravariantLifetime {
    _insn: jit_insn_t
});
impl<'a> Copy for Instruction<'a> {}

impl<'a> Instruction<'a> {
	/// Get the opcode of the instruction
	fn get_opcode(self) -> i32 {
		unsafe {
			jit_insn_get_opcode(self._insn)
		}
	}
	/// Get the destination value
	fn get_dest(self) -> Option<Value<'a>> {
		unsafe {
			from_ptr(jit_insn_get_dest(self._insn))
		}
	}
	/// Get if the destination value is a value
	fn dest_is_value(self) -> bool {
		unsafe {
			jit_insn_dest_is_value(self._insn) != 0
		}
	}
	/// Get the left value
	fn get_value1(self) -> Option<Value<'a>> {
		unsafe {
			from_ptr(jit_insn_get_value1(self._insn))
		}
	}
	/// Get the right value
	fn get_value2(self) -> Option<Value<'a>> {
		unsafe {
			from_ptr(jit_insn_get_value2(self._insn))
		}
	}
	/// Get the function containing this value
	fn get_function(self) -> Option<AnyFunction<'a>> {
		unsafe {
			from_ptr(jit_insn_get_function(self._insn))
		}
	}
	/// Get the signature of this value
	fn get_signature(self) -> Option<Type> {
		unsafe {
			from_ptr(jit_insn_get_signature(self._insn))
		}
	}
	/// Get the name of the instruction
	fn get_name(self) -> &'a str {
		unsafe {
			let name = jit_insn_get_name(self._insn);
			let name: &*const i8 = mem::transmute(&name);
			str::from_utf8(ffi::c_str_to_bytes(name)).unwrap()
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
	marker: ContravariantLifetime<'a>
}
impl<'a> Iterator for InstructionIter<'a> {
	type Item = Instruction<'a>;
	fn next(&mut self) -> Option<Instruction<'a>> {
		unsafe {
			from_ptr(jit_insn_iter_next(&mut self._iter))
		}
	}
}

/// Represents a single LibJIT block
native_ref!(Block ContravariantLifetime {
    _block: jit_block_t
});
impl<'a> Copy for Block<'a> {}
impl<'a> Block<'a> {
	/// Get the function containing this block
	pub fn get_function(self) -> AnyFunction<'a> {
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
				marker: ContravariantLifetime::<'a>
			}
		}
	}
}