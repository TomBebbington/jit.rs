#![feature(macro_rules)]
#[macro_export]
macro_rules! jit(
    (struct {
        $($name:expr: $ty:ty),*
    }) => ({
        let structure = Type::create_struct([
            $(get::<$ty>()),*
        ].as_mut_slice());
        structure.set_names(&[$($name),*]);
        structure
    });
    (union {
        $($name:expr: $ty:ty),+
    }) => ({
        let structure = Type::create_union([
            $(get::<$ty>()),+
        ].as_mut_slice());
        structure.set_names(&[$($name),+]);
        structure
    });
    ($func:ident, return) => (
        $func.insn_default_return()
    );
    ($func:ident, return $($t:tt)+) => (
        $func.insn_return(jit!($($t)+))
    );
    ($func:ident, $var:ident += $num:expr) => (
        $func.insn_store($var, &$func.insn_add($var, $num));
    );
    ($func:ident, $var:ident -= $num:expr) => (
        $func.insn_store($var, &$func.insn_sub($var, $num));
    );
    ($func:ident, $var:ident *= $num:expr) => (
        $func.insn_store($var, &$func.insn_mul($var, $num));
    );
    ($func:ident, $var:ident /= $num:expr) => (
        $func.insn_store($var, &$func.insn_div($var, $num));
    );
    ($func:ident, $a:expr + $b:expr) => (
        $func.insn_add($a, $b)
    );
    ($func:ident, $a:expr - $b:expr) => (
        $func.insn_sub($a, $b)
    );
    ($func:ident, $a:expr * $b:expr) => (
        $func.insn_mul($a, $b)
    );
    ($func:ident, $a:expr / $b:expr) => (
        $func.insn_div($a, $b)
    );
    ($func:ident, $a:expr % $b:expr) => (
        $func.insn_rem($a, $b)
    );
    ($func:ident, $var:ident = $val:expr) => (
        $func.insn_store($var, $val);
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
    ($func:expr, $value:expr) => (
        $func.insn_of(&$value)
    );
);
#[macro_export]
macro_rules! jit_func(
    ($ctx:expr, $func:ident, fn $name:ident($($arg:ident:$ty:ty),*) -> $ret:ty $value:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [$(get::<$ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0u;
            $(let ref $arg = $func[{i += 1; i - 1}];)+
            $value
        })
    });
    ($ctx:expr, $func:ident, fn $name:ident($($arg:ident:$arg_ty:ty),*) -> $ret:ty $value:expr, |$comp_func:ident| $comp:expr) => ({
        use std::default::Default;
        let sig = Type::create_signature(Default::default(), get::<$ret>(), [$(get::<$arg_ty>()),*].as_mut_slice());
        $ctx.build_func(sig, |$func| {
            let mut i = 0u;
            $(let ref $arg = $func[{i += 1; i - 1}];)*
            $value
        }).with::<($($arg_ty),*), $ret, _>(|$comp_func|
            $comp)
    });
);