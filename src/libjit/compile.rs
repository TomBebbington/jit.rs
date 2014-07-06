use bindings::{
    jit_type_void_ptr,
    jit_type_ubyte,
    jit_type_sbyte,
    jit_type_ushort,
    jit_type_short,
    jit_type_uint,
    jit_type_int,
    jit_type_nuint,
    jit_type_nint,
    jit_type_float32,
    jit_type_float64,
    jit_type_long,
    jit_type_ulong,
    jit_type_sys_bool,
    jit_type_sys_char,

    jit_value_create_long_constant,
    jit_value_create_nint_constant,
    jit_value_create_float64_constant,
    jit_value_create_float32_constant
};
use function::{
    CDECL,
    Function
};
use get_type = types::get;
use libc::c_long;
use value::Value;
use std::c_str::CString;
use std::mem::transmute;
use types::Type;
use util::NativeRef;
/// A type that can be compiled into a LibJIT representation
pub trait Compile {
    /// Get a JIT representation of this value
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a>;
    /// Get the JIT type repr of the value
    fn jit_type(_:Option<Self>) -> Type;
}
impl Compile for () {
    fn compile(&self, func:&Function) -> Value {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                jit_type_void_ptr,
                0
            ))
        }
    }
    #[inline]
    fn jit_type(_:Option<()>) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_void_ptr)
        }
    }
}
compile_prim!(f64, jit_type_float64, jit_value_create_float64_constant)
compile_prim!(f32, jit_type_float32, jit_value_create_float32_constant)
compile_prim!(int, jit_type_nint, jit_value_create_nint_constant, c_long)
compile_prim!(uint,jit_type_nuint, jit_value_create_nint_constant, c_long)
compile_prim!(i64, jit_type_long, jit_value_create_long_constant, c_long)
compile_prim!(u64, jit_type_ulong, jit_value_create_long_constant, c_long)
compile_prim!(i32, jit_type_int, jit_value_create_nint_constant, c_long)
compile_prim!(u32, jit_type_uint, jit_value_create_nint_constant, c_long)
compile_prim!(i16, jit_type_short, jit_value_create_nint_constant, c_long)
compile_prim!(u16, jit_type_ushort, jit_value_create_nint_constant, c_long)
compile_prim!(i8, jit_type_sbyte, jit_value_create_nint_constant, c_long)
compile_prim!(u8, jit_type_ubyte, jit_value_create_nint_constant, c_long)
compile_prim!(bool, jit_type_sys_bool, jit_value_create_nint_constant, c_long)
compile_prim!(char, jit_type_sys_char, jit_value_create_nint_constant, c_long)
impl Compile for *const u8 {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let c_str = unsafe { CString::new(transmute(*self), false) };
        let ty = jit!(&u8);
        let ptr = Value::new(func, ty);
        let length = c_str.len() + 1u;
        func.insn_store(&ptr, &func.insn_alloca(&func.insn_of(&length)));
        for enum_char in c_str.iter().enumerate() {
            let (pos, ch) = enum_char;
            let char_v = ch.compile(func);
            func.insn_store_relative(&ptr, pos as int, &char_v);
        }
        func.insn_store_relative(&ptr, c_str.len() as int, &func.insn_of(&'\0'));
        ptr
    }
    #[inline(always)]
    fn jit_type(_:Option<*const u8>) -> Type {
        ::get::<&u8>()
    }
}
impl<T:Compile> Compile for *mut T {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                jit!(&T).as_ptr(),
                self.to_uint() as c_long
            ))
        }
    }
    #[inline(always)]
    fn jit_type(_:Option<*mut T>) -> Type {
        jit!(&T)
    }
}
impl<'s> Compile for CString {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let ty = jit!(CString);
        let val = Value::new(func, ty.clone());
        let string:*const u8 = unsafe { transmute(self.as_ptr()) };
        func.insn_store_relative(&val, 0, &string.compile(func));
        func.insn_store_relative(&val, ty.find_name("is_owned").get_offset() as int, &true.compile(func));
        val
    }
    #[inline]
    fn jit_type(_:Option<CString>) -> Type {
        jit!(struct {
            "ptr": *const u8,
            "is_owned": bool
        })
    }
}
impl<'a> Compile for &'a str {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let str_ptr = {
            let ty = ::get::<*const u8>();
            let ptr = Value::new(func, ty);
            func.insn_store(&ptr, &func.insn_alloca(&func.insn_of(&self.len())));
            for enum_char in self.bytes().enumerate() {
                let (pos, ch) = enum_char;
                let char_v = ch.compile(func);
                func.insn_store_relative(&ptr, pos as int, &char_v);
            }
            ptr
        };
        let ty = ::get::<&'a str>();
        let val = Value::new(func, ty.clone());
        func.insn_store_relative(&val, 0, &str_ptr);
        func.insn_store_relative(&val, ty.find_name("len").get_offset() as int, &self.len().compile(func));
        val
    }
    #[inline]
    fn jit_type(_:Option<&'a str>) -> Type {
        jit!(struct {
            "ptr": *const u8,
            "len": uint
        })
    }
}
impl<'a> Compile for String {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let str_ptr = {
            let ty = ::get::<*const u8>();
            let ptr = Value::new(func, ty);
            func.insn_store(&ptr, &func.insn_alloca(&func.insn_of(&self.len())));
            for (pos, ch) in self.as_slice().bytes().enumerate() {
                let char_v = ch.compile(func);
                func.insn_store_relative(&ptr, pos as int, &char_v);
            }
            ptr
        };
        let ty = ::get::<String>();
        let val = Value::new(func, ty.clone());
        let length = self.len().compile(func);
        func.insn_store_relative(&val, 0, &length);
        func.insn_store_relative(&val, ty.clone().find_name("cap").get_offset() as int, &length);
        func.insn_store_relative(&val, ty.find_name("ptr").get_offset() as int, &str_ptr);
        val
    }
    #[inline]
    fn jit_type(_:Option<String>) -> Type {
        jit!(struct {
            "len": uint,
            "cap": uint,
            "ptr": &u8
        })
    }
}
impl<'a, T:Compile> Compile for Vec<T> {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let vec_ptr = {
            let ty = ::get::<&T>();
            let inner_ty = jit!(T);
            let ptr = Value::new(func, ty);
            let ptr_size = self.len() * inner_ty.get_size();
            let ptr_size = func.insn_of(&ptr_size);
            func.insn_store(&ptr, &func.insn_alloca(&ptr_size));
            for (pos, val) in self.iter().enumerate() {
                let val_v = val.compile(func);
                func.insn_store_relative(&ptr, pos as int, &val_v);
            }
            ptr
        };
        let ty = ::get::<String>();
        let val = Value::new(func, ty.clone());
        let length = func.insn_of(&self.len());
        func.insn_store_relative(&val, 0, &length);
        func.insn_store_relative(&val, ty.clone().find_name("cap").get_offset() as int, &length);
        func.insn_store_relative(&val, ty.find_name("ptr").get_offset() as int, &vec_ptr);
        val
    }
    #[inline]
    fn jit_type(_:Option<Vec<T>>) -> Type {
        jit!(struct {
            "len": uint,
            "cap": uint,
            "ptr": &T
        })
    }
}
impl<'a, T:Compile> Compile for &'a T {
    #[inline]
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        unsafe {
            NativeRef::from_ptr(jit_value_create_nint_constant(
                func.as_ptr(),
                jit!(T).as_ptr(),
                (*self as *const T).to_uint() as c_long
            ))
        }
    }
    #[inline]
    fn jit_type(_:Option<&'a T>) -> Type {
        Type::create_pointer(jit!(T))
    }
}
impl<R:Compile> Compile for fn() -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let ptr = (self as *const fn() -> R).to_uint().compile(func);
        func.insn_convert(&ptr, get_type::<fn() -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn() -> R>) -> Type {
        Type::create_signature(CDECL, jit!(R), &mut [])
    }
}
impl<A:Compile, R:Compile> Compile for fn(A) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let ptr = (self as *const fn(A) -> R).to_uint().compile(func);
        func.insn_convert(&ptr, get_type::<fn(A) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A) -> R>) -> Type {
        Type::create_signature(CDECL, jit!(R), &mut [jit!(A)])
    }
}
impl<A:Compile, B:Compile, R:Compile> Compile for fn(A, B) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let ptr = (self as *const fn(A, B) -> R).to_uint().compile(func);
        func.insn_convert(&ptr, get_type::<fn(A, B) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A, B) -> R>) -> Type {
        Type::create_signature(CDECL, jit!(R), &mut [jit!(A), jit!(B)])
    }
}
impl<A:Compile, B:Compile, C:Compile, R:Compile> Compile for fn(A, B, C) -> R {
    fn compile<'a>(&self, func:&Function<'a>) -> Value<'a> {
        let ptr = (self as *const fn(A, B, C) -> R).to_uint().compile(func);
        func.insn_convert(&ptr, get_type::<fn(A, B, C) -> R>(), false)
    }
    #[inline]
    fn jit_type(_:Option<fn(A, B, C) -> R>) -> Type {
        Type::create_signature(CDECL, jit!(R), &mut [jit!(A), jit!(B), jit!(C)])
    }
}