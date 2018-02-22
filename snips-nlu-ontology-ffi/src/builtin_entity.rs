#![allow(non_camel_case_types)]

use errors::*;
use ffi_utils::{CResult, CStringArray};
use libc;
use serde_json;
use snips_nlu_ontology::{BuiltinEntityKind, Language};
use std::convert::From;
use std::ffi::{CStr, CString};
use std::slice;
use std::str::FromStr;

pub trait CBuiltinEntity: From<::BuiltinEntity> {
    fn drop(&mut self);
}

#[repr(C)]
#[derive(Debug)]
pub struct CFullBuiltinEntity {
    pub entity: ::CSlotValue,
    pub entity_kind: *const libc::c_char,
    pub value: *const libc::c_char,
    pub range_start: libc::int32_t,
    pub range_end: libc::int32_t,
}

impl CBuiltinEntity for CFullBuiltinEntity {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.value as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.entity_kind as *mut libc::c_char) };
    }
}

impl From<::BuiltinEntity> for CFullBuiltinEntity {
    fn from(e: ::BuiltinEntity) -> CFullBuiltinEntity {
        Self {
            entity: ::CSlotValue::from(e.entity),
            entity_kind: CString::new(e.entity_kind.identifier()).unwrap().into_raw(),
            value: CString::new(e.value).unwrap().into_raw(), // String can not contains 0
            range_start: e.range.start as libc::int32_t,
            range_end: e.range.end as libc::int32_t,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CLightBuiltinEntity {
    pub entity: *const libc::c_char,
    pub entity_kind: *const libc::c_char,
    pub value: *const libc::c_char,
    pub range_start: libc::int32_t,
    pub range_end: libc::int32_t,
}

impl CBuiltinEntity for CLightBuiltinEntity {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.value as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.entity as *mut libc::c_char) };
        let _ = unsafe { CString::from_raw(self.entity_kind as *mut libc::c_char) };
    }
}

impl From<::BuiltinEntity> for CLightBuiltinEntity {
    fn from(e: ::BuiltinEntity) -> CLightBuiltinEntity {
        Self {
            entity: CString::new(serde_json::to_string(&e.entity).unwrap()).unwrap().into_raw(),
            entity_kind: CString::new(e.entity_kind.identifier()).unwrap().into_raw(),
            value: CString::new(e.value).unwrap().into_raw(), // String can not contains 0
            range_start: e.range.start as libc::int32_t,
            range_end: e.range.end as libc::int32_t,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CBuiltinEntityArray<T> {
    pub data: *const T,
    pub size: libc::int32_t, // Note: we can't use `libc::size_t` because it's not supported by JNA
}

impl<T> CBuiltinEntityArray<T> {
    pub fn from(input: Vec<T>) -> Self {
        Self {
            size: input.len() as libc::int32_t,
            data: Box::into_raw(input.into_boxed_slice()) as *const T,
        }
    }
}

impl<T> Drop for CBuiltinEntityArray<T> {
    fn drop(&mut self) {
        let _ = unsafe {
            Box::from_raw(slice::from_raw_parts_mut(
                self.data as *mut T,
                self.size as usize,
            ))
        };
    }
}

// We are forced to wrap this Box because lazy_static! require to be Sync but
// ffi's type `*const libc::c_char` isn't
struct DummyWrapper(Box<[*const libc::c_char]>);

unsafe impl Sync for DummyWrapper {}

#[no_mangle]
pub extern "C" fn nlu_ontology_all_builtin_entities() -> CStringArray {
    lazy_static! {
        static ref ALL: DummyWrapper = {
            DummyWrapper(
                BuiltinEntityKind::all()
                    .iter()
                    .map(|l| l.identifier().to_string())
                    .map(|l| CString::new(l).unwrap().into_raw() as *const libc::c_char)
                    .collect::<Vec<*const libc::c_char>>()
                    .into_boxed_slice()
            )
        };
    }

    CStringArray {
        data: ALL.0.as_ptr() as *const *const libc::c_char,
        size: ALL.0.len() as libc::int32_t,
    }
}

#[no_mangle]
pub extern "C" fn nlu_ontology_supported_builtin_entities(
    language: *const libc::c_char,
    results: *mut *const CStringArray,
) -> CResult {
    wrap!(get_supported_builtin_entities(language, results))
}

fn get_supported_builtin_entities(
    language: *const libc::c_char,
    results: *mut *const CStringArray,
) -> OntologyResult<()> {
    let language_str = unsafe { CStr::from_ptr(language) }.to_str()?;
    let language = Language::from_str(&*language_str.to_uppercase())?;
    let entities = BuiltinEntityKind::all()
        .iter()
        .filter(|e| e.supported_languages().contains(&language))
        .map(|e| e.identifier().to_string())
        .collect::<Vec<String>>();
    let c_entities = CStringArray::from(entities);
    unsafe {
        *results = Box::into_raw(Box::new(c_entities));
    }
    Ok(())
}
