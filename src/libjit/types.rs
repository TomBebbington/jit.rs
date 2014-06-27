use bindings::*;
use compile::Compile;
use function::ABI;
use libc::c_uint;
use std::kinds::marker::{
    ContravariantLifetime,
    Managed
};
use std::str::raw::from_c_str;
use std::c_str::ToCStr;
use util::NativeRef;
/// The types that a value can be
bitflags!(
    flags TypeKind: i32 {
        static Invalid      = -1,
        static Void         = 0,
        static SByte        = 1,
        static UByte        = 2,
        static Short        = 3,
        static UShort       = 4,
        static Int          = 5,
        static UInt         = 6,
        static NInt         = 7,
        static NUInt        = 8,
        static Long         = 9,
        static ULong        = 10,
        static Float32      = 11,
        static Float64      = 12,
        static NFloat       = 13,
        static MaxPrimitive = 13,
        static Struct       = 14,
        static Union        = 15,
        static Signature    = 16,
        static Pointer      = 17,
        static FirstTagged  = 32,
        static SysBool      = 10009,
        static SysChar      = 10010
    }
)
/// A type field iterator
pub struct Fields<'a> {
    _type: jit_type_t,
    index: c_uint,
    length: c_uint,
    marker: ContravariantLifetime<'a>
}
impl<'a> Fields<'a> {
    #[inline]
    fn new(ty:&'a Type) -> Fields<'a> {
        unsafe {
            Fields {
                _type: ty.as_ptr(),
                index: 0 as c_uint,
                length: jit_type_num_fields(ty.as_ptr()),
                marker: ContravariantLifetime::<'a>
            }
        }
    }
}
impl<'a> Iterator<(String, Type)> for Fields<'a> {
    fn next(&mut self) -> Option<(String, Type)> {
        unsafe {
            let index = self.index;
            self.index += 1;
            if index < self.length {
                let name = from_c_str(jit_type_get_name(self._type, index));
                let native_field = jit_type_get_field(self._type, index);
                if name.len() == 0 || native_field.is_null() {
                    None
                } else {
                    let field = NativeRef::from_ptr(native_field);
                    Some((name, field))
                }
            } else {
                None
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        ((self.length - self.index) as uint, None)
    }
    #[inline]
    fn count(&mut self) -> uint {
        let count = self.length - self.index;
        self.index = self.length;
        count as uint
    }
    #[inline]
    fn nth(&mut self, n:uint) -> Option<(String, Type)> {
        self.index += n as u32;
        self.next()
    }
}
/**
 * An object that represents a native system type.
 *
 * Each `Type` represents a basic system type,
 * be it a primitive, a struct, a union, a pointer, or a function signature.
 * The library uses this information to lay out values in memory.
*/
#[deriving(PartialEq)]
pub struct Type {
    _type: jit_type_t,
    marker: Managed
}
impl NativeRef for Type {
    #[inline]
    unsafe fn as_ptr(&self) -> jit_type_t {
        self._type
    }
    #[inline]
    unsafe fn from_ptr(ptr:jit_type_t) -> Type {
        Type {
            _type: ptr,
            marker: Managed
        }
    }
}
impl Clone for Type {
    #[inline]
    /// Make a copy of the type descriptor by increasing its reference count.
    fn clone(&self) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_copy(self.as_ptr()))
        }
    }
}
#[unsafe_destructor]
impl Drop for Type {
    #[inline]
    /**
     * Free a type descriptor by decreasing its reference count.
     *
     * This function is save to use on pre-defined types, which
     * are never actually freed.
     */
    fn drop(&mut self) {
        unsafe {
            jit_type_free(self.as_ptr());
        }
    }
}
impl Type {
    /// Create a type descriptor for a function signature.
    pub fn create_signature(abi: ABI, return_type: &Type, params: &mut [&Type]) -> Type {
        unsafe {
            let mut native_params:Vec<jit_type_t> = params.iter().map(|param| param.as_ptr()).collect();
            let signature = jit_type_create_signature(abi as jit_abi_t, return_type.as_ptr(), native_params.as_mut_ptr(), params.len() as c_uint, 1);
            NativeRef::from_ptr(signature)
        }
    }

    fn create_complex(fields: &mut [&Type], union: bool) -> Type {
        unsafe {
            let mut native_fields:Vec<jit_type_t> = fields.iter().map(|field| field.as_ptr()).collect();
            let f = if union { jit_type_create_union } else { jit_type_create_struct };
            let ty:jit_type_t = f(native_fields.as_mut_ptr(), fields.len() as c_uint, 1);
            NativeRef::from_ptr(ty)
        }
    }
    /// Create a type descriptor for a structure.
    pub fn create_struct(fields: &mut [&Type]) -> Type {
        Type::create_complex(fields, false)
    }
    /// Create a type descriptor for a union.
    pub fn create_union(fields: &mut [&Type]) -> Type {
        let inner = Type::create_complex(fields, true);
        Type::create_struct(&mut [&get::<int>(), &inner])
    }
    /// Create a type descriptor for a pointer to another type.
    pub fn create_pointer(pointee: &Type) -> Type {
        unsafe {
            let ptr = jit_type_create_pointer(pointee.as_ptr(), 1);
            NativeRef::from_ptr(ptr)
        }
    }
    #[inline]
    /// Get the size of this type in bytes.
    pub fn get_size(&self) -> jit_nuint {
        unsafe {
            jit_type_get_size(self.as_ptr())
        }
    }
    /**
     * Get a value that indicates the kind of this type. This allows
     * callers to quickly classify a type to determine how it should
     * be handled further.
     */
    #[inline]
    pub fn get_kind(&self) -> TypeKind {
        unsafe {
            TypeKind::from_bits(jit_type_get_kind(self.as_ptr())).unwrap()
        }
    }
    #[inline]
    /// Get the type that is referred to by this pointer type.
    pub fn get_ref(&self) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_get_ref(self.as_ptr()))
        }
    }
    /// Set the field or parameter names of this type.
    pub fn set_names<T:ToCStr>(&self, names:&[T]) -> bool {
        unsafe {
            let native_names : Vec<*i8> = names.iter().map(|name| name.to_c_str().unwrap()).collect();
            jit_type_set_names(self.as_ptr(), native_names.as_ptr() as *mut *mut i8, names.len() as u32) != 0
        }
    }
    #[inline]
    /// Iterator over the type's fields
    pub fn iter_fields<'a>(&'a self) -> Fields<'a> {
        Fields::new(self)
    }
    #[inline]
    /// Find the field/parameter index for a particular name.
    pub fn find_name<T:ToCStr>(&self, name:T) -> uint {
        name.with_c_str(|c_name| unsafe {
            jit_type_find_name(self.as_ptr(), c_name) as uint
        })
    }
}
#[inline]
/// Get the Rust type given as a type descriptor
pub fn get<T:Compile>() -> Type {
    Compile::jit_type(None::<T>)
}