use raw::*;
use compile::Compile;
use function::Abi;
use libc::{c_uint, c_void};
use std::marker::{ContravariantLifetime, NoCopy};
use std::{fmt, mem};
use std::fmt::Display;
use std::ffi::{self, CString};
use util::{self, from_ptr, NativeRef};
pub use kind::TypeKind;
/// The integer representation of a type
pub mod kind {
    use libc::c_int;
    bitflags!(
        flags TypeKind: c_int {
            const Void = 0,
            const SByte = 1,
            const UByte = 2,
            const Short = 3,
            const UShort = 4,
            const Int = 5,
            const UInt = 6,
            const NInt = 7,
            const NUInt = 8,
            const Long = 9,
            const ULong = 10,
            const Float32 = 11,
            const Float64 = 12,
            const NFloat = 13,
            const Struct = 14,
            const Union = 15,
            const Signature = 16,
            const Pointer = 17,
            const FirstTagged = 2,
            const SysBool = 10009,
            const SysChar = 10010
        }
    );
}
impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let kind = self.get_kind();
        if kind.contains(kind::Pointer) {
            try!(fmt.write_str("*mut "));
            self.get_ref().unwrap().fmt(fmt)
        } else if kind.contains(kind::Signature) {
            try!("fn(".fmt(fmt));
            for arg in self.params() {
                try!(arg.fmt(fmt));
            }
            try!(") -> ".fmt(fmt));
            match self.get_return() {
                Some(x) => x.fmt(fmt),
                None => "()".fmt(fmt)
            }
        } else {
            write!(fmt, "{}", try!(util::dump(|fd| {
                unsafe { jit_dump_type(mem::transmute(fd), self.as_ptr()) };
            })))
        }
    }
}
pub mod consts {
    use raw::*;
    use types::Type;
    builtin_types!{
        jit_type_void -> VOID;
        jit_type_sbyte -> SBYTE;
        jit_type_ubyte -> UBYTE;
        jit_type_short -> SHORT;
        jit_type_ushort -> USHORT;
        jit_type_int -> INT;
        jit_type_uint -> UINT;
        jit_type_nint -> NINT;
        jit_type_nuint -> NUINT;
        jit_type_long -> LONG;
        jit_type_ulong -> ULONG;
        jit_type_float32 -> FLOAT32;
        jit_type_float64 -> FLOAT64;
        jit_type_nfloat -> NFLOAT;
        jit_type_void_ptr -> VOID_PTR;
        jit_type_sys_bool -> SYS_BOOL;
        jit_type_sys_char -> SYS_CHAR;
        jit_type_sys_uchar -> SYS_UCHAR;
        jit_type_sys_short -> SYS_SHORT;
        jit_type_sys_ushort -> SYS_USHORT;
        jit_type_sys_int -> SYS_INT;
        jit_type_sys_uint -> SYS_UINT;
        jit_type_sys_long -> SYS_LONG;
        jit_type_sys_ulong -> SYS_ULONG;
        jit_type_sys_longlong -> SYS_LONGLONG;
        jit_type_sys_ulonglong -> SYS_ULONGLONG;
        jit_type_sys_float -> SYS_FLOAT;
        jit_type_sys_double -> SYS_DOUBLE;
        jit_type_sys_long_double -> SYS_LONG_DOUBLE
    }
}
/// A single field of a struct
#[deriving(PartialEq)]
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
                let bytes = ffi::c_str_to_bytes(&c_name);
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
        }
    }
    #[inline(always)]
    /// Get the type of the field
    pub fn get_type(&self) -> Type {
        unsafe {
            from_ptr(jit_type_get_field(self._type, self.index))
        }
    }
    #[inline(always)]
    /// Get the offset of the field
    pub fn get_offset(&self) -> usize {
        unsafe {
            jit_type_get_offset(self._type, self.index) as usize
        }
    }
}
/// Iterates through all the fields of a struct
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
impl<'a> Iterator for Fields<'a> {
    type Item = Field<'a>;
    fn next(&mut self) -> Option<Field<'a>> {
        if self.index < self.length {
            let index = self.index;
            self.index += 1;
            Some(Field {
                index: index,
                _type: self._type,
                marker: ContravariantLifetime::<'a>
            })
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.length - self.index) as usize, None)
    }
}
/// Iterator through all the arguments a function takes
pub struct Params<'a> {
    _type: jit_type_t,
    index: c_uint,
    length: c_uint,
    marker: ContravariantLifetime<'a>
}
impl<'a> Params<'a> {
    fn new(ty:Type) -> Params<'a> {
        unsafe {
            Params {
                _type: ty.as_ptr(),
                index: 0,
                length: jit_type_num_params(ty.as_ptr()),
                marker: ContravariantLifetime::<'a>
            }
        }
    }
}
impl<'a> Iterator for Params<'a> {
    type Item = Type;
    fn next(&mut self) -> Option<Type> {
        if self.index < self.length {
            let index = self.index;
            self.index += 1;
            unsafe { from_ptr(jit_type_get_param(self._type, index)) }
        } else {
            None
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.length - self.index) as usize, None)
    }
}
/// A static type that is owned by LibJIT itself
pub trait StaticType: Copy {
    /// Get type contained in this static type
    fn get(self) -> Type;
}
/// An object that represents a native system type.
/// Each `Type` represents a basic system type, be it a primitive, a struct, a
/// union, a pointer, or a function signature. The library uses this information
/// to lay out values in memory.
/// Types are not attached to a context so they are reference-counted by LibJIT,
/// so internally they are represented as `Rc<TypeData>`.
#[derive(PartialEq, Eq)]
pub struct Type {
    _type: jit_type_t,
    no_copy: NoCopy
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
            no_copy: NoCopy
        }
    }
}
impl Clone for Type {
    #[inline]
    /// Make a copy of the type descriptor by increasing its reference count.
    fn clone(&self) -> Type {
        unsafe {
            from_ptr(jit_type_copy(self.as_ptr()))
        }
    }
}
#[unsafe_destructor]
impl Drop for Type {
    #[inline(always)]
    /// Free a type descriptor by decreasing its reference count.
    /// This function is safe to use on pre-defined types, which are never
    /// actually freed.
    fn drop(&mut self) {
        unsafe {
            jit_type_free(self.as_ptr());
        }
    }
}
extern fn free_data<T:'static>(data: *mut c_void) {
    unsafe {
        let actual_data:Box<T> = mem::transmute(data);
        mem::drop(actual_data);
    }
}
impl Type {
    /// Create a type descriptor for a function signature.
    pub fn create_signature(abi: Abi, return_type: Type, params: &mut [Type]) -> Type {
        unsafe {
            let mut native_params:Vec<jit_type_t> = params.iter().map(|param| param.as_ptr()).collect();
            let signature = jit_type_create_signature(abi as jit_abi_t, return_type.as_ptr(), native_params.as_mut_ptr(), params.len() as c_uint, 1);
            from_ptr(signature)
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a structure.
    pub fn create_struct(fields: &mut [Type]) -> Type {
        unsafe {
            let mut native_fields:Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
            from_ptr(jit_type_create_struct(native_fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a union.
    pub fn create_union(fields: &mut [Type]) -> Type {
        unsafe {
            let mut native_fields:Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
            from_ptr(jit_type_create_union(native_fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a pointer to another type.
    pub fn create_pointer(pointee: Type) -> Type {
        unsafe {
            let ptr = jit_type_create_pointer(pointee.as_ptr(), 1);
            from_ptr(ptr)
        }
    }
    #[inline(always)]
    /// Create a new tagged type
    pub fn create_tagged<T:'static>(ty:Type, kind: kind::TypeKind, data: Box<T>) -> Type {
        unsafe {
            let free_data:extern fn(*mut c_void) = free_data::<T>;
            let ty = jit_type_create_tagged(ty.as_ptr(), kind.bits(), mem::transmute(&*data), Some(free_data), 1);
            mem::forget(data);
            from_ptr(ty)
        }
    }
    #[inline(always)]
    /// Get the size of this type in bytes.
    pub fn get_size(&self) -> usize {
        unsafe {
            jit_type_get_size(self.as_ptr()) as usize
        }
    }
    #[inline(always)]
    /// Get the alignment of this type in bytes.
    pub fn get_alignment(&self) -> usize {
        unsafe {
            jit_type_get_alignment(self.as_ptr()) as usize
        }
    }
    #[inline]
    /// Get a value that indicates the kind of this type. This allows callers to
    /// quickly classify a type to determine how it should be handled further.
    pub fn get_kind(&self) -> kind::TypeKind {
        unsafe {
            mem::transmute(jit_type_get_kind(self.as_ptr()))
        }
    }
    #[inline(always)]
    /// Get the type that is referred to by this pointer type.
    pub fn get_ref(&self) -> Option<Type> {
        unsafe {
            from_ptr(jit_type_get_ref(self.as_ptr()))
        }
    }

    #[inline(always)]
    pub fn get_tagged_data<T:'static>(&self) -> Option<&T> {
        unsafe {
            mem::transmute(jit_type_get_tagged_data(self.as_ptr()))
        }
    }
    #[inline(always)]
    pub fn set_tagged_data<T:'static>(&self, data: Box<T>) {
        unsafe {
            let free_data:extern fn(*mut c_void) = free_data::<T>;
            jit_type_set_tagged_data(self.as_ptr(), mem::transmute(&*data), Some(free_data));
            mem::forget(data);
        }
    }
    #[inline(always)]
    /// Get the type returned by this function type.
    pub fn get_return(&self) -> Option<Type> {
        unsafe {
            from_ptr(jit_type_get_return(self.as_ptr()))
        }
    }
    /// Set the field or parameter names of this type.
    pub fn set_names(&self, names:&[&str]) -> bool {
        unsafe {
            let names = names.iter().map(|name| CString::from_slice(name.as_bytes())).collect::<Vec<_>>();
            let mut c_names = names.iter().map(|name| mem::transmute(name.as_ptr())).collect::<Vec<_>>();
            jit_type_set_names(self.as_ptr(), c_names.as_mut_ptr(), names.len() as u32) != 0
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
    pub fn find_name<'a>(&'a self, name:&str) -> Field<'a> {
        unsafe {
            let c_name = CString::from_slice(name.as_bytes());
            Field {
                index: jit_type_find_name(self.as_ptr(), mem::transmute(c_name.as_ptr())),
                _type: self.as_ptr(),
                marker: ContravariantLifetime::<'a>
            }
        }
    }
    #[inline(always)]
    /// Check if this is a pointer
    pub fn is_primitive(&self) -> bool {
        unsafe {
            jit_type_is_primitive(self.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a struct
    pub fn is_struct(&self) -> bool {
        unsafe {
            jit_type_is_struct(self.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a union
    pub fn is_union(&self) -> bool {
        unsafe {
            jit_type_is_union(self.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a signature
    pub fn is_signature(&self) -> bool {
        unsafe {
            jit_type_is_signature(self.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a pointer
    pub fn is_pointer(&self) -> bool {
        unsafe {
            jit_type_is_pointer(self.as_ptr()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is tagged
    pub fn is_tagged(&self) -> bool {
        unsafe {
            jit_type_is_tagged(self.as_ptr()) != 0
        }
    }
}
#[inline(always)]
/// Get the Rust type given as a type descriptor
pub fn get<T: Compile>() -> Type {
    <T as Compile>::get_type()
}