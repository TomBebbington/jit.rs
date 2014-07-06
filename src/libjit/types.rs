use bindings::*;
use compile::Compile;
use function::ABI;
use libc::c_uint;
use std::kinds::marker::{
    ContravariantLifetime,
    Managed
};
use std::mem::transmute;
use std::str::raw::from_c_str;
use std::c_str::ToCStr;
use util::NativeRef;
#[repr(i32)]
#[deriving(Show)]
pub enum TypeKind {
    Void         = 0,
    SByte        = 1,
    UByte        = 2,
    Short        = 3,
    UShort       = 4,
    Int          = 5,
    UInt         = 6,
    NInt         = 7,
    NUInt        = 8,
    Long         = 9,
    ULong        = 10,
    Float32      = 11,
    Float64      = 12,
    NFloat       = 13,
    Struct       = 14,
    Union        = 15,
    Signature    = 16,
    Pointer      = 17,
    FirstTagged  = 32,
    SysBool      = 10009,
    SysChar      = 10010
}
/// A single field of a struct
pub struct Field<'a> {
    /// The index of the field
    pub index: c_uint,
    _type: jit_type_t,
    marker: ContravariantLifetime<'a>
}
impl<'a> Field<'a> {
    #[inline]
    /// Get the field's name or none if it lacks one
    pub fn get_name(&self) -> Option<String> {
        unsafe {
            let c_name = jit_type_get_name(self._type, self.index);
            if c_name.is_null() {
                None
            } else {
                Some(from_c_str(c_name))
            }
        }
    }
    #[inline]
    /// Get the type of the field
    pub fn get_type(&self) -> Type {
        unsafe {
            NativeRef::from_ptr(jit_type_get_field(self._type, self.index))
        }
    }
    #[inline]
    /// Get the offset of the field
    pub fn get_offset(&self) -> uint {
        unsafe {
            jit_type_get_offset(self._type, self.index) as uint
        }
    }
}
/// A type field iterator
pub struct Fields<'a> {
    _type: jit_type_t,
    index: c_uint,
    length: c_uint,
    marker: ContravariantLifetime<'a>
}
impl<'a> Fields<'a> {
    #[inline(always)]
    fn new(ty:Type) -> Fields<'a> {
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
impl<'a> Iterator<Field<'a>> for Fields<'a> {
    fn next(&mut self) -> Option<Field<'a>> {
        let index = self.index;
        self.index += 1;
        if index < self.length {
            Some(Field {
                _type: self._type.clone(),
                index: index,
                marker: ContravariantLifetime::<'a>
            })
        } else {
            None
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
    fn nth(&mut self, n:uint) -> Option<Field<'a>> {
        self.index += n as c_uint;
        self.next()
    }
}
pub struct Params<'a> {
    _type: jit_type_t,
    index: c_uint,
    length: c_uint,
    marker: ContravariantLifetime<'a>
}
impl<'a> Params<'a> {
    #[inline(always)]
    fn new(ty:Type) -> Params<'a> {
        unsafe {
            Params {
                _type: ty.as_ptr(),
                index: 0 as c_uint,
                length: jit_type_num_params(ty.as_ptr()),
                marker: ContravariantLifetime::<'a>
            }
        }
    }
}
impl<'a> Iterator<Type> for Params<'a> {
    fn next(&mut self) -> Option<Type> {
        let index = self.index;
        self.index += 1;
        if index < self.length {
            Some(unsafe {
                NativeRef::from_ptr(jit_type_get_param(self._type, index))
            })
        } else {
            None
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
    fn nth(&mut self, n:uint) -> Option<Type> {
        self.index += n as c_uint;
        self.next()
    }
}
/// An object that represents a native system type.
/// Each `Type` represents a basic system type, be it a primitive, a struct, a
/// union, a pointer, or a function signature. The library uses this information
/// to lay out values in memory.
#[deriving(PartialEq)]
pub struct Type {
    _type: jit_type_t,
    marker: Managed
}
impl NativeRef for Type {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> jit_type_t {
        self._type
    }
    #[inline(always)]
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
    /// Free a type descriptor by decreasing its reference count.
    /// This function is safe to use on pre-defined types, which are never
    /// actually freed.
    fn drop(&mut self) {
        unsafe {
            jit_type_free(self.as_ptr());
        }
    }
}
impl Type {
    /// Create a type descriptor for a function signature.
    pub fn create_signature(abi: ABI, return_type: Type, params: &mut [Type]) -> Type {
        unsafe {
            let mut native_params:Vec<jit_type_t> = params.iter().map(|param| param.as_ptr()).collect();
            let signature = jit_type_create_signature(abi as jit_abi_t, return_type.as_ptr(), native_params.as_mut_ptr(), params.len() as c_uint, 1);
            NativeRef::from_ptr(signature)
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a structure.
    pub fn create_struct(fields: &mut [Type]) -> Type {
        unsafe {
            let mut native_fields:Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
            NativeRef::from_ptr(jit_type_create_struct(native_fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a union.
    pub fn create_union(fields: &mut [Type]) -> Type {
        unsafe {
            let mut native_fields:Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
            NativeRef::from_ptr(jit_type_create_union(native_fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a pointer to another type.
    pub fn create_pointer(pointee: Type) -> Type {
        unsafe {
            let ptr = jit_type_create_pointer(pointee.as_ptr(), 1);
            NativeRef::from_ptr(ptr)
        }
    }
    #[inline]
    /// Get the size of this type in bytes.
    pub fn get_size(&self) -> uint {
        unsafe {
            jit_type_get_size(self.as_ptr()) as uint
        }
    }
    #[inline]
    /// Get a value that indicates the kind of this type. This allows
    /// callers to quickly classify a type to determine how it should
    /// be handled further.
    pub fn get_kind(&self) -> TypeKind {
        unsafe {
            transmute(jit_type_get_kind(self.as_ptr()))
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
            let native_names : Vec<*const i8> = names.iter().map(|name| name.to_c_str().unwrap()).collect();
            jit_type_set_names(self.as_ptr(), native_names.as_ptr() as *mut *mut i8, names.len() as u32) != 0
        }
    }
    #[inline(always)]
    /// Iterator over the type's fields
    pub fn fields<'a>(&'a self) -> Fields<'a> {
        Fields::new(self.clone())
    }
    #[inline(always)]
    /// Iterator over the function signature's parameters
    pub fn params<'a>(&'a self) -> Params<'a> {
        Params::new(self.clone())
    }
    #[inline]
    /// Find the field/parameter index for a particular name.
    pub fn find_name<'b, T:ToCStr>(&'b self, name:T) -> Field<'b> {
        name.with_c_str(|c_name| unsafe {
            Field {
                index: jit_type_find_name(self.as_ptr(), c_name),
                _type: self.as_ptr(),
                marker: ContravariantLifetime::<'b>
            }
        })
    }
    #[inline(always)]
    /// Check if this is a pointer
    pub fn is_pointer(&self) -> bool {
        unsafe {
            jit_type_is_pointer(self.as_ptr()) != 0
        }
    }
}
#[inline(always)]
/// Get the Rust type given as a type descriptor
pub fn get<T:Compile>() -> Type {
    Compile::jit_type(None::<T>)
}