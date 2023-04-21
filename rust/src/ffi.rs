#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(clippy::all)]
#![allow(rustdoc::broken_intra_doc_links)]

use libloading::Library;
// Include generated bindings file
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Bridgestan {
    pub fn into_library(self) -> Library {
        self.__library
    }
}
