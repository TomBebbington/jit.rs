#[macro_export]
/// Construct a JIT struct with the fields given
macro_rules! jit_struct(
    ($($name:ident: $ty:ty),*) => ({
        let structure = Type::create_struct([
            $(get::<$ty>()),*
        ].as_mut_slice());
        structure.set_names(&[$(stringify!($name)),*]);
        structure
    });
    ($($ty:ty),+ ) => ({
        Type::create_struct([
            $(get::<$ty>()),+
        ].as_mut_slice())
    })
);

#[macro_export]
/// Construct a JIT union with the fields given
macro_rules! jit_union(
    ($($name:ident: $ty:ty),*) => ({
        let structure = Type::create_union([
            $(get::<$ty>()),*
        ].as_mut_slice());
        structure.set_names(&[$(stringify!($name)),*]);
        structure
    });
    ($($ty:ty),+ ) => ({
        Type::create_union([
            $(get::<$ty>()),+
        ].as_mut_slice())
    })
);
#[macro_export]
/// Construct a JIT function signature with the arguments and return type given
macro_rules! jit_fn(
    ($($arg:ty),* -> $ret:ty) => ({
        use std::default::Default;
        Type::create_signature(Default::default(), get::<$ret>(), [
            $(get::<$arg>()),*
        ].as_mut_slice())
    });
    (raw $($arg:expr),* -> $ret:expr) => ({
        use std::default::Default;
        Type::create_signature(Default::default(), $ret, [
            $($arg),*
        ].as_mut_slice())
    });
);

#[macro_export]
macro_rules! jit(
    ($func:ident, return) => (
        $func.insn_default_return()
    );
    ($func:ident, return $($t:tt)+) => (
        $func.insn_return(jit!($func, $($t)+))
    );
    ($func:ident, $var:ident += $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_add($var, jit!($func, $($t)+)));
    );
    ($func:ident, $var:ident -= $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_sub($var, jit!($func, $($t)+)));
    );
    ($func:ident, $var:ident *= $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_mul($var, jit!($func, $($t)+)));
    );
    ($func:ident, $var:ident /= $($t:tt)+) => (
        $func.insn_store($var, &$func.insn_div($var, jit!($func, $($t)+)));
    );
    ($func:ident, $($a:tt)+ + $($b:tt)+) => (
        $func.insn_add(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ - $($b:tt)+) => (
        $func.insn_sub(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ * $($b:tt)+) => (
        $func.insn_mul(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ / $($b:tt)+) => (
        $func.insn_div(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, $($a:tt)+ % $($b:tt)+) => (
        $func.insn_rem(jit!($func, $($a)+), jit!($func, $($b)+))
    );
    ($func:ident, ($($t:tt)+).sqrt()) => (
        $func.insn_sqrt(&jit!($func, $($t)+))
    );
    ($func:ident, $var:ident = $($t:tt)+) => (
        $func.insn_store($var, jit($func, $val));
    );
    ($func:ident, *$var:ident) => (
        $func.insn_load($var)
    );
    ($func:ident, call($call:expr,
        $($arg:expr),+
    )) => (
        $func.insn_call(None::<String>, $call, None, [$($arg),+].as_mut_slice())
    );
    ($func:ident, jump_table($value:expr,
        $($label:ident),+
    )) => (
    let ($($label),+) = {
        $(let $label:Label = Label::new($func);)+
        $func.insn_jump_table($value, [
            $($label),+
        ].as_mut_slice());
        ($($label),+)
    });
);
#[macro_export]
macro_rules! jit_func(
    ($ctx:expr, $func:ident, $name:ident() -> $ret:ty, $value:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [].as_mut_slice());
        $ctx.build_func(sig, |$func| $value)
    });
    ($ctx:expr, $func:ident, $name:ident($($arg:ident:$ty:ty),+) -> $ret:ty, $value:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [$(get::<$arg_ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0u;
            $(let $arg = {
                i += 1;
                $func[i - 1]
            };)*
            $value
        })
    });
    ($ctx:expr, $func:ident, $name:ident() -> $ret:ty, $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [].as_mut_slice());
        $ctx.build_func(sig, |$func| $value)
            .with::<(), $ret, _>(|$comp_func| $comp)
    });
    ($ctx:expr, $func:ident, $name:ident($($arg:ident:$arg_ty:ty),+) -> $ret:ty, $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [$(get::<$arg_ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0us;
            $(let $arg = {
                i += 1;
                $func[i - 1]
            };)*
            $value
        }).with::<($($arg_ty),*), $ret, _>(|$comp_func|
            $comp)
    });
);