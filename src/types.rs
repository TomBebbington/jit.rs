use raw::*;
use compile::Compile;
use function::Abi;
use alloc::oom;
use libc::{c_char, c_uint, c_void};
use util::{from_ptr, from_ptr_opt};
use std::borrow::*;
use std::marker::PhantomData;
use std::{fmt, mem, str};
use std::iter::IntoIterator;
use std::ffi::{self, CString};
use std::ops::{Deref, DerefMut};

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
        if kind.contains(TypeKind::SysChar) {
            fmt.write_str("char")
        } else if kind.contains(TypeKind::SysBool) {
            fmt.write_str("bool")
        } else if kind.contains(TypeKind::Pointer) {
            try!(fmt.write_str("&mut"));
            write!(fmt, "&mut {:?}", self.get_ref().unwrap())
        } else if kind.contains(TypeKind::Signature) {
            try!(fmt.write_str("fn("));
            let params = self.params();
            let (size, _) = params.size_hint();
            for (i, arg) in params.enumerate() {
                try!(write!(fmt, "{:?}", arg));
                if i < size - 1 {
                    try!(fmt.write_str(", "));
                }
            }
            try!(fmt.write_str(")"));
            if let Some(x) = self.get_return() {
                if !x.get_kind().contains(TypeKind::Void) {
                    try!(write!(fmt, " -> {:?}", x))
                }
            }
            Ok(())
        } else if kind.contains(TypeKind::Struct) {
            try!(fmt.write_str("("));
            let fields = self.fields();
            let (size, _) = fields.size_hint();
            for (i, field) in fields.enumerate() {
                try!(write!(fmt, "{:?}", field.get_type()));
                if i < size - 1 {
                    try!(fmt.write_str(", "));
                }
            }
            fmt.write_str(")")
        } else if kind.contains(TypeKind::Union) {
            try!(fmt.write_str("union("));
            let fields = self.fields();
            let (size, _) = fields.size_hint();
            for (i, field) in fields.enumerate() {
                try!(write!(fmt, "{:?}", field.get_type()));
                if i < size - 1 {
                    try!(fmt.write_str(", "));
                }
            }
            fmt.write_str(")")
        } else if kind.contains(TypeKind::NFloat) {
            fmt.write_str("float")
        } else if kind.contains(TypeKind::Float32) {
            fmt.write_str("f32")
        } else if kind.contains(TypeKind::Float64) {
            fmt.write_str("f64")
        } else if kind.contains(TypeKind::ULong) {
            fmt.write_str("u64")
        } else if kind.contains(TypeKind::Long) {
            fmt.write_str("i64")
        } else if kind.contains(TypeKind::NUInt) {
            fmt.write_str("usize")
        } else if kind.contains(TypeKind::NInt) {
            fmt.write_str("isize")
        } else if kind.contains(TypeKind::UInt) {
            fmt.write_str("u32")
        } else if kind.contains(TypeKind::Int) {
            fmt.write_str("i32")
        } else if kind.contains(TypeKind::UShort) {
            fmt.write_str("u16")
        } else if kind.contains(TypeKind::Short) {
            fmt.write_str("i16")
        } else if kind.contains(TypeKind::UByte) {
            fmt.write_str("u8")
        } else if kind.contains(TypeKind::SByte) {
            fmt.write_str("i8")
        } else {
            fmt.write_str("()")
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
    use util::from_ptr;
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
                let c_name = ffi::CStr::from_ptr(c_name);
                Some(str::from_utf8(c_name.to_bytes()).unwrap())
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
                _type: ty.into(),
                index: 0,
                length: jit_type_num_fields(ty.into()),
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
                _type: ty.into(),
                index: 0,
                length: jit_type_num_params(ty.into()),
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
            unsafe { from_ptr_opt(jit_type_get_param(self._type, index)) }
        } else {
            None
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        ((self.length - self.index) as usize, None)
    }
}
/// An object that represents a native system type
///
/// Each `&Ty` represents a basic system type, be it a primitive, a struct, a
/// union, a pointer, or a function signature. The library uses this information
/// to lay out values in memory.
///
/// Types are not attached to a context so they are reference-counted by LibJIT,
/// so internally they are represented as `Rc<Ty>`. This represents a reference
/// to the inner `Ty`.
pub struct Ty(PhantomData<[()]>);
native_ref!(&Ty = jit_type_t);
impl ToOwned for Ty {
    type Owned = Type;
    fn to_owned(&self) -> Type {
        unsafe {
            from_ptr(jit_type_copy(self.into()))
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
///
/// Each `Type` represents a basic system type, be it a primitive, a struct, a
/// union, a pointer, or a function signature. The library uses this information
/// to lay out values in memory.
///
/// Types are not attached to a context so they are reference-counted by LibJIT,
/// so internally they are represented as `Rc<Ty>`.
#[derive(PartialEq, Eq)]
pub struct Type {
    _type: jit_type_t,
}
native_ref!(Type, _type: jit_type_t);
impl Clone for Type {
    #[inline]
    /// Make a copy of the type descriptor by increasing its reference count.
    fn clone(&self) -> Type {
        unsafe {
            from_ptr(jit_type_copy((&**self).into()))
        }
    }
}
impl Drop for Type {
    #[inline(always)]
    /// Free a type descriptor by decreasing its reference count.
    fn drop(&mut self) {
        unsafe {
            jit_type_free(self.into());
        }
    }
}
impl<'a> Deref for Type {
    type Target = Ty;
    fn deref(&self) -> &Ty {
        unsafe {
            mem::transmute(self._type)
        }
    }
}
impl<'a> DerefMut for Type {
    fn deref_mut(&mut self) -> &mut Ty {
        unsafe {
            mem::transmute(self._type)
        }
    }
}
/// A copy-on-write type
pub type CowType<'a> = Cow<'a, Ty>;
/// A static type
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
            let mut params:&mut [jit_type_t] = mem::transmute(params);
            let signature = jit_type_create_signature(abi as jit_abi_t, return_type.into(), params.as_mut_ptr(), params.len() as c_uint, 1);
            from_ptr(signature)
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a structure.
    pub fn new_struct(fields: &mut [&Ty]) -> Type {
        unsafe {
            let fields:&mut [jit_type_t] = mem::transmute(fields);
            from_ptr(jit_type_create_struct(fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a union.
    pub fn new_union(fields: &mut [&Ty]) -> Type {
        unsafe {
            let fields:&mut [jit_type_t] = mem::transmute(fields);
            from_ptr(jit_type_create_union(fields.as_mut_ptr(), fields.len() as c_uint, 1))
        }
    }
    #[inline(always)]
    /// Create a type descriptor for a pointer to another type.
    pub fn new_pointer(pointee: &Ty) -> Type {
        unsafe {
            let ptr = jit_type_create_pointer(pointee.into(), 1);
            from_ptr(ptr)
        }
    }
}
impl Ty {
    #[inline(always)]
    /// Get the size of this type in bytes.
    pub fn get_size(&self) -> usize {
        unsafe {
            jit_type_get_size(self.into()) as usize
        }
    }
    #[inline(always)]
    /// Get the alignment of this type in bytes.
    pub fn get_alignment(&self) -> usize {
        unsafe {
            jit_type_get_alignment(self.into()) as usize
        }
    }
    #[inline]
    /// Get a value that indicates the kind of this type. This allows callers to
    /// quickly classify a type to determine how it should be handled further.
    pub fn get_kind(&self) -> kind::TypeKind {
        unsafe {
            mem::transmute(jit_type_get_kind(self.into()))
        }
    }
    #[inline(always)]
    /// Get the type that is referred to by this pointer type.
    pub fn get_ref(&self) -> Option<&Ty> {
        unsafe {
            from_ptr_opt(jit_type_get_ref(self.into()))
        }
    }
    #[inline(always)]
    /// Get the type returned by this function type.
    pub fn get_return(&self) -> Option<&Ty> {
        unsafe {
            //let ty = jit_type_get_return(self.into())
            from_ptr_opt(jit_type_get_return(self.into()))
        }
    }
    /// Set the field or parameter names of this struct or union type.
    ///
    /// ```rust
    /// use jit::*;
    /// let f64_t = get::<f64>();
    /// let mut ty = Type::new_struct(&mut [&f64_t, &f64_t]);
    /// ty.set_names(&["x", "y"]);
    /// assert!(ty.get_field("x").get_type() == &f64_t as &Ty);
    /// assert!(ty.get_field("y").get_type() == &f64_t as &Ty);
    /// ```
    pub fn set_names(&mut self, names: &[&str]) {
        unsafe {
            let names = names.iter()
                             .map(|name| CString::new(name.as_bytes()).unwrap())
                             .collect::<Vec<_>>();
            let mut c_names = names.iter()
                            .map(|name| name.as_bytes().as_ptr() as *mut c_char)
                            .collect::<Vec<_>>();
            if jit_type_set_names(self.into(), c_names.as_mut_ptr(), names.len() as u32) == 0 {
                oom();
            }
        }
    }
    #[inline(always)]
    /// Iterate over the type's fields
    pub fn fields(&self) -> Fields {
        Fields::new(self)
    }
    #[inline(always)]
    /// Iterate over the function signature's parameters
    pub fn params(&self) -> Params {
        Params::new(self)
    }
    #[inline]
    /// Find the field/parameter index for a particular name.
    pub fn get_field(&self, name:&str) -> Field {
        unsafe {
            let c_name = CString::new(name.as_bytes()).unwrap();
            Field {
                index: jit_type_find_name(self.into(), c_name.as_bytes().as_ptr() as *const c_char),
                _type: self.into(),
                marker: PhantomData,
            }
        }
    }
    #[inline(always)]
    /// Check if this is a pointer
    pub fn is_primitive(&self) -> bool {
        unsafe {
            jit_type_is_primitive(self.into()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a struct
    pub fn is_struct(&self) -> bool {
        unsafe {
            jit_type_is_struct(self.into()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a union
    pub fn is_union(&self) -> bool {
        unsafe {
            jit_type_is_union(self.into()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a signature
    pub fn is_signature(&self) -> bool {
        unsafe {
            jit_type_is_signature(self.into()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is a pointer
    pub fn is_pointer(&self) -> bool {
        unsafe {
            jit_type_is_pointer(self.into()) != 0
        }
    }
    #[inline(always)]
    /// Check if this is tagged
    pub fn is_tagged(&self) -> bool {
        unsafe {
            jit_type_is_tagged(self.into()) != 0
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
    _marker: PhantomData<T>
}
impl<'a, T> Into<jit_type_t> for &'a TaggedType<T> {
    /// Convert into a native pointer
    fn into(self) -> jit_type_t {
        self._type
    }
}
impl<T> From<jit_type_t> for TaggedType<T> {
    /// Convert from a native pointer
    fn from(ptr: jit_type_t) -> TaggedType<T> {
        TaggedType {
            _type: ptr,
            _marker: PhantomData
        }
    }
}
impl<T> TaggedType<T> {
    /// Create a new tagged type
    pub fn new(ty:&Ty, kind: kind::TypeKind, data: Box<T>) -> TaggedType<T> {
        unsafe {
            let free_data:extern fn(*mut c_void) = ::free_data::<T>;
            let ty = jit_type_create_tagged(ty.into(), kind.bits(), mem::transmute(&*data), Some(free_data), 1);
            mem::forget(data);
            from_ptr(ty)
        }
    }
    /// Get the data this is tagged to
    pub fn get_tagged_data(&self) -> Option<&T> {
        unsafe {
            mem::transmute(jit_type_get_tagged_data(self.into()))
        }
    }
    /// Get the type this is tagged to
    pub fn get_tagged_type(&self) -> &Ty {
        unsafe {
            from_ptr(jit_type_get_tagged_type(self.into()))
        }
    }
    /// Change the data this is tagged to
    pub fn set_tagged_data(&self, data: Box<T>) {
        unsafe {
            let free_data:extern fn(*mut c_void) = ::free_data::<T>;
            jit_type_set_tagged_data(self.into(), mem::transmute(&*data), Some(free_data));
            mem::forget(data);
        }
    }
}
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
pub fn get<'a, T>() -> CowType<'a> where T:Compile<'a> {
    <T as Compile>::get_type()
}
