extern crate libjit_sys;

use libjit_sys::*;

/// An [example from the libjit manual][example], translated to Rust.  We
/// compile and call a function equivalent to:
///
/// ```c
/// int mul_add(int x, int y, int z)
/// {
///     return x * y + z;
/// }
/// ```
///
/// This is unsafe FFI code, written directly against libjit's C API.
///
/// [example]: http://www.gnu.org/software/libjit/doc/libjit_3.html#Tutorials
unsafe fn mul_add_tutorial() {
    let ctx = jit_context_create();

    jit_context_build_start(ctx);

    let mut params = vec!();
    params.push(jit_type_int);
    params.push(jit_type_int);
    params.push(jit_type_int);
    
    let sig = jit_type_create_signature(jit_abi_cdecl, jit_type_int,
                                        params.as_mut_ptr(),
                                        params.len() as u32, 1);

    let func = jit_function_create(ctx, sig);

    let x = jit_value_get_param(func, 0);
    let y = jit_value_get_param(func, 1);
    let z = jit_value_get_param(func, 2);
    
    let temp1 = jit_insn_mul(func, x, y);
    let temp2 = jit_insn_add(func, temp1, z);
    jit_insn_return(func, temp2);

    jit_function_compile(func);
    jit_context_build_end(ctx);

    let arg_values: Vec<jit_int> = vec!(3, 5, 2);
    let arg_ptrs: Vec<*const jit_int> =
        arg_values.iter().map(|p| p as *const jit_int).collect();
    let mut result: jit_int = 0;

    jit_function_apply(func,
                       // We can probably use a smaller hammer than
                       // std::mem::translate.
                       std::mem::transmute(arg_ptrs.as_ptr()),
                       std::mem::transmute(&mut result));

    println!("mul_add(3, 5, 2) = {}", result);

    jit_context_destroy(ctx);
}

fn main() {
    unsafe { mul_add_tutorial() }
}
