use raw::*;
use compile::Compile;
use function::Abi;
use alloc::oom;
use libc::{c_uint, c_void};
use std::borrow::*;
use std::marker::PhantomData;
use std::{fmt, mem, str};
use std::iter::IntoIterator;
use std::fmt::Display;
use std::ffi::{self, CString};
use std::ops::Deref;
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
impl fmt::Debug for Ty {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let kind = self.get_kind();
        if kind.contains(kind::Pointer) {
            try!(fmt.write_str("*mut "));
            self.get_ref().unwrap().fmt(fmt)
        } else if kind.contains(kind::Signature) {
            try!(fmt.write_str("fn("));
            for arg in self.params() {
                try!(write!(fmt, "{:?}", arg));
            }
            try!(fmt.write_str(")"));
            match self.get_return() {
                Some(x) => try!(write!(fmt, "{:?}", x)),
                None => ()
            }
            Ok(())
        } else {
            write!(fmt, "{}", try!(util::dump(|fd| {
                unsafe { jit_dump_type(mem::transmute(fd), self.as_ptr()) };
            })))
        }
    }
}
impl fmt::Debug for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), fmt)
    }
}
/// Type constants
pub mod consts {
    use raw::*;
    use types::StaticType;
    builtin_types!{
        jit_type_void -> get_void;
        jit_type_sbyte -> get_sbyte;
        jit_type_ubyte -> get_ubyte;
        jit_type_short -> get_short;
        jit_type_ushort -> get_ushort;
        jit_type_int -> get_int;
        jit_type_uint -> get_uint;
        jit_type_nint -> get_nint;
        jit_type_nuint -> get_nuint;
        jit_type_long -> get_long;
        jit_type_ulong -> get_ulong;
        jit_type_float32 -> get_float32;
        jit_type_float64 -> get_float64;
        jit_type_nfloat -> get_nfloat;
        jit_type_void_ptr -> get_void_ptr;
        jit_type_sys_bool -> get_sys_bool;
        jit_type_sys_char -> get_sys_char;
        jit_type_sys_uchar -> get_sys_uchar;
        jit_type_sys_short -> get_sys_short;
        jit_type_sys_ushort -> get_sys_ushort;
        jit_type_sys_int -> get_sys_int;
        jit_type_sys_uint -> get_sys_uint;
        jit_type_sys_long -> get_sys_long;
        jit_type_sys_ulong -> get_sys_ulong;
        jit_type_sys_longlong -> get_sys_longlong;
        jit_type_sys_ulonglong -> get_sys_ulonglong;
        jit_type_sys_float -> get_sys_float;
        jit_type_sys_double -> get_sys_double;
        jit_type_sys_long_double -> get_sys_long_double
    }
}
/// A single field of a struct
#[derive(PartialEq)]
pub struct Field<'a> {
    /// The index of the field
    pub index: c_uint,
    _type: jit_type_t,
    marker: PhantomData<&'a ()>,
}
impl<'a> Field<'a> {
    #[inline]
    /// Get the field's name or none if it lacks one
    pub fn get_name(&self) -> Option<&'a str> {
        unsafe {
            let c_name = jit_type_get_name(self._type, self.index);
            if c_name.is_null() {
                None
            } else {
                Some(str::from_utf8(ffi::CStr::from_ptr(c_name).to_bytes()).unwrap())
            }
        }
    }
    #[inline(always)]
    /// Get the type of the field
    pub fn get_type(&self) -> &'a Ty {
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
    marker: PhantomData<&'a ()>,
}
impl<'a> Fields<'a> {
    #[inline(always)]
    fn new(ty:&'a Ty) -> Fields<'a> {
        unsafe {
            Fields {
                _type: ty.as_ptr(),
                index: 0,
                length: jit_type_num_fields(ty.as_ptr()),
                marker: PhantomData,
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
                marker: PhantomData,
            })
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.length - self.index) as usize, None)
    }
}
//deref owned type into type ref
/// Iterator through all the arguments a function takes
pub struct Params<'a> {
    _type: jit_type_t,
    index: c_uint,
    length: c_uint,
    marker: PhantomData<&'a ()>
}
impl<'a> Params<'a> {
    fn new(ty:&'a Ty) -> Params<'a> {
        unsafe {
            Params {
                _type: ty.as_ptr(),
                index: 0,
                length: jit_type_num_params(ty.as_ptr()),
                marker: PhantomData,
            }
        }
    }
}
impl<'a> Iterator for Params<'a> {
    type Item = &'a Ty;
    fn next(&mut self) -> Option<&'a Ty> {
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
/// An object that represents a native system type.
/// This represents a basic system type, be it a primitive, a struct, a
/// union, a pointer, or a function signature. The library uses this information
/// to lay out values in memory.
#[derive(Eq, PartialEq)]
pub struct Ty;
impl<'a> NativeRef for &'a Ty {
    unsafe fn as_ptr(&self) -> jit_type_t {
        mem::transmute(self)
    }
    unsafe fn from_ptr(ptr: jit_type_t) -> &'a Ty {
        mem::transmute(ptr)
    }
}
impl ToOwned for Ty {
    type Owned = Type;
    fn to_owned(&self) -> Type {
        unsafe {
            from_ptr(jit_type_copy(self.as_ptr()))
        }
    }
}
impl Borrow<Ty> for Type {
    fn borrow(&self) -> &Ty {
        unsafe {
            mem::transmute(self._type)
        }
    }
}

/// An owned object that represents a native system type.
/// Each `Type` represents a basic system type, be it a primitive, a struct, a
/// union, a pointer, or a function signature. The library uses this information
/// to lay out values in memory.
/// Types are not attached to a context so they are reference-counted by LibJIT,
/// so internally they are represented as `Rc<Ty>`.
#[derive(PartialEq, Eq)]
pub struct Type {
    _type: jit_type_t,
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
    fn drop(&mut self) {
        unsafe {
            jit_type_free(self.as_ptr());
        }
    }
}
impl<'a> Deref for Type {
    type Target = Ty;
    fn deref(&self) -> &Ty {
        unsafe {
            mem::transmute(&self._type)
        }
    }
}
pub type CowType<'a> = Cow<'a, Ty>;
pub type StaticType = &'static Ty;
impl Into<CowType<'static>> for Type {
    fn into(self) -> CowType<'static> {
        Cow::Owned(self)
    }
}
impl<'a> Into<CowType<'a>> for &'a Ty {
    fn into(self) -> CowType<'a> {
        Cow::Borrowed(self)
    }
}
impl Type {
    /// Create a type descriptor for a function signature.
    pub fn new_signature(abi: Abi, return_type: &Ty, params: &mut [&Ty]) -> Type {
        unsafe {
            let mut native_params:Vec<jit_type_t> = params.iter().map(|param| param.as_ptr()).collect();
            let signature = jit_type_create_signature(abi as jit_abi_t, return_type.as_ptr(), native_params.as_mut_ptr(), params.len() as c_uint, 1);
            from_ptr(signature)
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a structure.
    pub fn new_struct(fields: &mut [&Ty]) -> Type {
        unsafe {
            let mut native_fields:Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
            from_ptr(jit_type_create_struct(native_fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a union.
    pub fn new_union(fields: &mut [&Ty]) -> Type {
        unsafe {
            let mut native_fields:Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
            from_ptr(jit_type_create_union(native_fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a pointer to another type.
    pub fn new_pointer(pointee: &Ty) -> Type {
        unsafe {
            let ptr = jit_type_create_pointer(pointee.as_ptr(), 1);
            from_ptr(ptr)
        }
    }
}
impl Ty {
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
    pub fn get_ref(&self) -> Option<&Ty> {
        unsafe {
            from_ptr(jit_type_get_ref(self.as_ptr()))
        }
    }
    #[inline(always)]
    /// Get the type returned by this function type.
    pub fn get_return(&self) -> Option<&Ty> {
        unsafe {
            from_ptr(jit_type_get_return(self.as_ptr()))
        }
    }
    /// Set the field or parameter names of this type.
    pub fn set_names(&self, names: &[&str]) {
        unsafe {
            let names = names.iter()
                             .map(|name| CString::new(name.as_bytes()).unwrap())
                             .collect::<Vec<_>>();
            let mut c_names = names.iter()
                            .map(|name| mem::transmute(name.as_ptr()))
                            .collect::<Vec<_>>();
            if jit_type_set_names(self.as_ptr(), c_names.as_mut_ptr(), names.len() as u32) == 0 {
                oom();
            }
        }
    }
    #[inline(always)]
    /// Iterator over the type's fields
    pub fn fields(&self) -> Fields {
        Fields::new(self)
    }
    #[inline(always)]
    /// Iterator over the function signature's parameters
    pub fn params(&self) -> Params {
        Params::new(self)
    }
    #[inline]
    /// Find the field/parameter index for a particular name.
    pub fn get_field(&self, name:&str) -> Field {
        unsafe {
            let c_name = CString::new(name.as_bytes()).unwrap();
            Field {
                index: jit_type_find_name(self.as_ptr(), mem::transmute(c_name.as_ptr())),
                _type: self.as_ptr(),
                marker: PhantomData,
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
impl<'a> IntoIterator for &'a Ty {
    type IntoIter = Fields<'a>;
    type Item = Field<'a>;
    fn into_iter(self) -> Fields<'a> {
        self.fields()
    }
}

#[derive(PartialEq, Eq)]
pub struct TaggedType<T> {
    _type: jit_type_t,
    _marker: PhantomData<fn(T) -> T>
}
impl<T> NativeRef for TaggedType<T> {
    #[inline(always)]
    unsafe fn as_ptr(&self) -> jit_type_t {
        self._type
    }
    #[inline(always)]
    unsafe fn from_ptr(ptr:jit_type_t) -> TaggedType<T> {
        TaggedType {
            _type: ptr,
            _marker: PhantomData,
        }
    }
}
impl<T> TaggedType<T> where T:'static {
    /// Create a new tagged type
    pub fn new(ty:&Ty, kind: kind::TypeKind, data: Box<T>) -> TaggedType<T> {
        unsafe {
            let free_data:extern fn(*mut c_void) = ::free_data::<T>;
            let ty = jit_type_create_tagged(ty.as_ptr(), kind.bits(), mem::transmute(&*data), Some(free_data), 1);
            mem::forget(data);
            from_ptr(ty)
        }
    }
    /// Get the data this is tagged to
    pub fn get_tagged_data(&self) -> Option<&T> {
        unsafe {
            mem::transmute(jit_type_get_tagged_data(self.as_ptr()))
        }
    }
    /// Get the type this is tagged to
    pub fn get_tagged_type(&self) -> &Ty {
        unsafe {
            from_ptr(jit_type_get_tagged_type(self.as_ptr()))
        }
    }
    /// Change the data this is tagged to
    pub fn set_tagged_data(&self, data: Box<T>) {
        unsafe {
            let free_data:extern fn(*mut c_void) = ::free_data::<T>;
            jit_type_set_tagged_data(self.as_ptr(), mem::transmute(&*data), Some(free_data));
            mem::forget(data);
        }
    }
}
#[unsafe_destructor]
impl<T> Drop for TaggedType<T> {
    #[inline(always)]
    /// Free a type descriptor by decreasing its reference count.
    /// This function is safe to use on pre-defined types, which are never
    /// actually freed.
    fn drop(&mut self) {
        unsafe {
            jit_type_free(self._type);
        }
    }
}
impl<T> Deref for TaggedType<T> {
    type Target = Ty;
    fn deref(&self) -> &Ty {
        unsafe {
            mem::transmute(self._type)
        }
    }
}
#[inline(always)]
/// Get the Rust type given as a type descriptor
pub fn get<T>() -> CowType<'static> where T:Compile {
    <T as Compile>::get_type()
}
