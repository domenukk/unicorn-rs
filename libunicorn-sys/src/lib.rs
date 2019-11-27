#![deny(rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod unicorn_const;
pub mod arm64_const;
pub mod arm_const;
pub mod m68k_const;
pub mod mips_const;
pub mod sparc_const;
pub mod x86_const;

use crate::unicorn_const::{Arch, Error, HookType, Mode, Query, Protection};
use core::{fmt, slice};
use libc::{c_char, c_int, c_void};

#[allow(non_camel_case_types)]
pub type uc_handle = libc::size_t;
#[allow(non_camel_case_types)]
pub type uc_hook = libc::size_t;
#[allow(non_camel_case_types)]
pub type uc_context = libc::size_t;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MemRegion {
    /// The start address of the region (inclusive).
    pub begin: u64,
    /// The end address of the region (inclusive).
    pub end: u64,
    /// The memory permissions of the region.
    pub perms: Protection,
}

extern "C" {
    pub fn uc_version(major: *mut u32, minor: *mut u32) -> u32;
    pub fn uc_arch_supported(arch: Arch) -> bool;
    pub fn uc_open(arch: Arch, mode: Mode, engine: *mut uc_handle) -> Error;
    pub fn uc_close(engine: uc_handle) -> Error;
    pub fn uc_free(mem: libc::size_t) -> Error;
    pub fn uc_errno(engine: uc_handle) -> Error;
    pub fn uc_strerror(error_code: Error) -> *const c_char;
    pub fn uc_reg_write(engine: uc_handle, regid: c_int, value: *const c_void) -> Error;
    pub fn uc_reg_read(engine: uc_handle, regid: c_int, value: *mut c_void) -> Error;
    pub fn uc_mem_write(
        engine: uc_handle,
        address: u64,
        bytes: *const u8,
        size: libc::size_t,
    ) -> Error;
    pub fn uc_mem_read(
        engine: uc_handle,
        address: u64,
        bytes: *mut u8,
        size: libc::size_t,
    ) -> Error;
    pub fn uc_mem_map(engine: uc_handle, address: u64, size: libc::size_t, perms: u32) -> Error;
    pub fn uc_mem_map_ptr(
        engine: uc_handle,
        address: u64,
        size: libc::size_t,
        perms: u32,
        ptr: *mut c_void,
    ) -> Error;
    pub fn uc_mem_unmap(engine: uc_handle, address: u64, size: libc::size_t) -> Error;
    pub fn uc_mem_protect(engine: uc_handle, address: u64, size: libc::size_t, perms: u32)
        -> Error;
    pub fn uc_mem_regions(
        engine: uc_handle,
        regions: *const *const MemRegion,
        count: *mut u32,
    ) -> Error;
    pub fn uc_emu_start(
        engine: uc_handle,
        begin: u64,
        until: u64,
        timeout: u64,
        count: libc::size_t,
    ) -> Error;
    pub fn uc_emu_stop(engine: uc_handle) -> Error;
    pub fn uc_hook_add(
        engine: uc_handle,
        hook: *mut uc_hook,
        hook_type: HookType,
        callback: libc::size_t,
        user_data: *mut libc::size_t,
        begin: u64,
        end: u64,
        ...
    ) -> Error;
    pub fn uc_hook_del(engine: uc_handle, hook: uc_hook) -> Error;
    pub fn uc_query(engine: uc_handle, query_type: Query, result: *mut libc::size_t) -> Error;
    pub fn uc_context_alloc(engine: uc_handle, context: *mut uc_context) -> Error;
    pub fn uc_context_save(engine: uc_handle, context: uc_context) -> Error;
    pub fn uc_context_restore(engine: uc_handle, context: uc_context) -> Error;

    pub fn uc_afl_fuzz(
        engine: uc_handle, 
        input_file: *const u8,
        place_input_callback: libc::size_t, 
        exits: *const u64,
        exit_count: libc::size_t,
        validate_crash_callback: libc:: size_t,
        always_validate: bool,
        persistent_iters: u32,
        data: *const c_void
    ) -> unicorn_const::AflRet;

    pub fn uc_afl_forkserver_start(
        engine: uc_handle,
        exits: *const u64,
        exit_count: libc::size_t
    ) -> unicorn_const::AflRet;

    /* A start with "less features" for our afl use-case */
    /* this is largely copied from uc_emu_start, just without setting the entry point, counter and timeout. */
    pub fn uc_afl_emu_start(engine: uc_handle) -> Error;

    pub fn uc_afl_next(engine: uc_handle) -> unicorn_const::AflRet;

}

impl Error {
    pub fn msg(self) -> &'static str {
        unsafe {
            let ptr = uc_strerror(self);
            let len = libc::strlen(ptr);
            let s = slice::from_raw_parts(ptr as *const u8, len);
            // We believe that strings returned by `uc_strerror` are always valid ASCII chars.
            // Hence they also must be a valid Rust str.
            core::str::from_utf8_unchecked(s)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg().fmt(fmt)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
