extern crate libjit_sys;

use libjit_sys::*;

/// This is a simplified version of a libjit [example][].
///
/// [example]: http://www.gnu.org/software/libjit/doc/libjit_3.html#Tutorials
unsafe fn jit_add(x: jit_int, y: jit_int) -> jit_int {
    let ctx = jit_context_create();

    jit_context_build_start(ctx);

    let mut params = vec!();
    params.push(jit_type_int);
    params.push(jit_type_int);
    
    let sig = jit_type_create_signature(jit_abi_cdecl, jit_type_int,
                                        params.as_mut_ptr(),
                                        params.len() as u32, 1);

    let func = jit_function_create(ctx, sig);

    let x_param = jit_value_get_param(func, 0);
    let y_param = jit_value_get_param(func, 1);
    
    let temp = jit_insn_add(func, x_param, y_param);
    jit_insn_return(func, temp);

    jit_function_compile(func);
    jit_context_build_end(ctx);

    let arg_values: Vec<jit_int> = vec!(x, y);
    let arg_ptrs: Vec<*const jit_int> =
        arg_values.iter().map(|p| p as *const jit_int).collect();
    let mut result: jit_int = 0;

    jit_function_apply(func,
                       // We can probably use a smaller hammer than
                       // std::mem::translate.
                       std::mem::transmute(arg_ptrs.as_ptr()),
                       std::mem::transmute(&mut result));

    jit_context_destroy(ctx);

    return result;
}

#[test]
fn test_jit_basics() {
    assert_eq!(5, unsafe { jit_add(2, 3) });
}
