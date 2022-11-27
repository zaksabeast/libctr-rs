use crate::{
    ipc::{Command, CurrentProcessId, Handles, PermissionBuffer, StaticBuffer},
    memory::MemoryBlock,
    res::{error, CtrResult},
    srv::get_service_handle_direct,
    Handle,
};
use alloc::vec;
use core::{
    convert::TryInto,
    sync::atomic::{AtomicU32, Ordering},
};
use cstr_core::CString;
use no_std_io::{EndianRead, EndianWrite};
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DefaultRootCert {
    NintendoCa = 0x1,
    NintendoCaG2 = 0x2,
    NintendoCaG3 = 0x3,
    NintendoClass2Ca = 0x4,
    NintendoClass2CaG2 = 0x5,
    NintendoClass2CaG3 = 0x6,
    CyberTrust = 0x7,
    AddTrustExternalCa = 0x8,
    Comodo = 0x9,
    USERTrust = 0xA,
    DigiCertEv = 0xB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum RequestMethod {
    Get = 1,
    Post = 2,
    Head = 3,
    Put = 4,
    Delete = 5,
}

#[derive(IntoPrimitive)]
#[repr(u32)]
pub enum RequestStatus {
    Unknown = 0,
    RequestInProgress = 5,
    DownloadReady = 7,
}

impl From<u32> for RequestStatus {
    fn from(raw_status: u32) -> Self {
        match raw_status {
            5 => Self::RequestInProgress,
            7 => Self::DownloadReady,
            _ => Self::Unknown,
        }
    }
}

static HTTP_SERVICE_HANDLE: AtomicU32 = AtomicU32::new(0);
static HTTP_SHARED_MEMORY_HANDLE: AtomicU32 = AtomicU32::new(0);

pub(crate) fn get_httpc_service_raw_handle() -> u32 {
    HTTP_SERVICE_HANDLE.load(Ordering::Relaxed)
}

pub fn httpc_init(memory_block: MemoryBlock) -> CtrResult {
    let service_handle = get_service_handle_direct("http:C")?;

    httpc_initialize(
        &service_handle,
        memory_block.get_size(),
        memory_block.get_handle(),
    )?;

    // We need to drop this since the raw handles are stored in an atomic
    let dropped_service_handle = core::mem::ManuallyDrop::new(service_handle);
    let dropped_memory_block = core::mem::ManuallyDrop::new(memory_block);

    // This is safe since we're sending it to another process, not copying it
    let raw_service_handle = unsafe { dropped_service_handle.get_raw() };
    HTTP_SERVICE_HANDLE.store(raw_service_handle, Ordering::Relaxed);

    // This is safe since we're sending it to another process, not copying it
    let shared_memory_raw_handle = unsafe { dropped_memory_block.get_handle().get_raw() };
    HTTP_SHARED_MEMORY_HANDLE.store(shared_memory_raw_handle, Ordering::Relaxed);

    Ok(())
}

#[derive(EndianRead, EndianWrite)]
struct HttpcInitializeIn {
    shared_memory_block_size: u32,
    current_process_id: CurrentProcessId,
    memory_block_handles: Handles,
}

fn httpc_initialize(
    service_handle: &Handle,
    shared_memory_block_size: usize,
    shared_memory_block_handle: &Handle,
) -> CtrResult {
    let raw_shared_memory_block_handle = unsafe { shared_memory_block_handle.get_raw() };
    let input = HttpcInitializeIn {
        shared_memory_block_size: shared_memory_block_size.try_into()?,
        current_process_id: CurrentProcessId::new(),
        memory_block_handles: Handles::new(vec![raw_shared_memory_block_handle]),
    };
    let raw_handle = unsafe { service_handle.get_raw() };
    Command::new(0x10044, input).send(raw_handle)
}

#[derive(EndianRead, EndianWrite)]
struct InitializeConnectionSessionId {
    context_handle: u32,
    current_process_id: CurrentProcessId,
}

pub(crate) fn httpc_initialize_connection_session(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
) -> CtrResult {
    let input = InitializeConnectionSessionId {
        context_handle: unsafe { context_handle.get_raw() },
        current_process_id: CurrentProcessId::new(),
    };
    let raw_handle = unsafe { session_handle.get_raw() };
    Command::new(0x80042, input).send(raw_handle)
}

#[derive(EndianRead, EndianWrite)]
struct CreateContextIn {
    url_len: u32,
    method: u32,
    url: PermissionBuffer,
}

pub(crate) fn httpc_create_context(
    method: RequestMethod,
    url: &str,
) -> CtrResult<HttpContextHandle> {
    let c_url = CString::new(url)?;
    let url_bytes = c_url.as_bytes_with_nul();
    let url_len = url_bytes.len().try_into()?;

    let input = CreateContextIn {
        url_len,
        method: method as u32,
        url: PermissionBuffer::new_read(url_bytes),
    };
    let result: u32 = Command::new(0x20082, input).send(get_httpc_service_raw_handle())?;
    Ok(result.into())
}

pub(crate) fn httpc_set_proxy_default(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
) -> CtrResult {
    let raw_session_handle = unsafe { session_handle.get_raw() };
    let raw_context_handle = unsafe { context_handle.get_raw() };
    Command::new(0xE0040, raw_context_handle).send(raw_session_handle)
}

#[derive(EndianRead, EndianWrite)]
struct AddRequestHeaderFieldIn {
    context_handle: u32,
    header_name_len: u32,
    value_len: u32,
    header_name: StaticBuffer,
    value: PermissionBuffer,
}

pub(crate) fn httpc_add_request_header_field(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
    header_name: &str,
    value: &str,
) -> CtrResult {
    let c_header_name = CString::new(header_name)?;
    let header_name_bytes = c_header_name.as_bytes_with_nul();
    let header_name_len = header_name_bytes.len().try_into()?;

    let c_value = CString::new(value)?;
    let value_bytes = c_value.as_bytes_with_nul();
    let value_len = value_bytes.len().try_into()?;

    let raw_context_handle = unsafe { context_handle.get_raw() };
    let raw_session_handle = unsafe { session_handle.get_raw() };
    let input = AddRequestHeaderFieldIn {
        header_name_len,
        value_len,
        context_handle: raw_context_handle,
        header_name: StaticBuffer::new(header_name_bytes, 3),
        value: PermissionBuffer::new_read(value_bytes),
    };
    Command::new(0x1100C4, input).send(raw_session_handle)
}

#[derive(EndianRead, EndianWrite)]
struct AddPostDataAsciiIn {
    context_handle: u32,
    field_name_len: u32,
    value_len: u32,
    field_name: StaticBuffer,
    value: PermissionBuffer,
}

pub(crate) fn httpc_add_post_data_ascii(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
    post_field_name: &str,
    value: &str,
) -> CtrResult {
    if !value.is_ascii() {
        return Err(error::invalid_value());
    }

    let c_post_field_name = CString::new(post_field_name)?;
    let post_field_name_bytes = c_post_field_name.as_bytes_with_nul();
    let post_field_name_len = post_field_name_bytes.len().try_into()?;

    let c_value = CString::new(value)?;
    let value_bytes = c_value.as_bytes_with_nul();
    let value_len = value_bytes.len().try_into()?;

    let raw_context_handle = unsafe { context_handle.get_raw() };
    let raw_session_handle = unsafe { session_handle.get_raw() };
    let input = AddPostDataAsciiIn {
        value_len,
        context_handle: raw_context_handle,
        field_name_len: post_field_name_len,
        field_name: StaticBuffer::new(post_field_name_bytes, 3),
        value: PermissionBuffer::new_read(value_bytes),
    };
    Command::new(0x1200C4, input).send(raw_session_handle)
}

pub(crate) fn httpc_set_socket_buffer_size(
    session_handle: &Handle,
    socket_buffer_size: u32,
) -> CtrResult {
    let raw_session_handle = unsafe { session_handle.get_raw() };
    Command::new(0xA0040, socket_buffer_size).send(raw_session_handle)
}

#[derive(EndianRead, EndianWrite)]
struct ReceiveDataTimeoutIn {
    context_handle: u32,
    out_len: u32,
    nanosecond_timeout: u64,
    out: PermissionBuffer,
}

pub(crate) fn httpc_receive_data_with_timeout(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
    out_buffer: &mut [u8],
    nanosecond_timeout: u64,
) -> CtrResult {
    let raw_context_handle = unsafe { context_handle.get_raw() };
    let raw_session_handle = unsafe { session_handle.get_raw() };
    let input = ReceiveDataTimeoutIn {
        nanosecond_timeout,
        context_handle: raw_context_handle,
        out_len: out_buffer.len().try_into()?,
        out: PermissionBuffer::new_write(out_buffer),
    };
    Command::new(0xC0102, input).send(raw_session_handle)
}

pub(crate) fn httpc_begin_request(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
) -> CtrResult {
    let raw_context_handle = unsafe { context_handle.get_raw() };
    let raw_session_handle = unsafe { session_handle.get_raw() };
    Command::new(0x90040, raw_context_handle).send(raw_session_handle)
}

pub(crate) struct HttpContextHandle(u32);

impl HttpContextHandle {
    /// Returns the raw u32 handle
    /// # Safety
    /// Because a Handle closes itself when it's dropped, a raw handle might have been previously closed.
    /// The user must guarantee the handle will outlive the raw handle (and all copies/clones of the raw handle)
    ///
    /// Admittedly this is less of memory safety and more of logical safety, but since that's the purpose of this abstraction
    /// unsafe will be used in this way.
    pub(crate) unsafe fn get_raw(&self) -> u32 {
        self.0
    }
}

impl From<u32> for HttpContextHandle {
    fn from(raw_handle: u32) -> Self {
        Self(raw_handle)
    }
}

impl Drop for HttpContextHandle {
    // If this doesn't close, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        Command::new(0x30040, self.0).send::<()>(get_httpc_service_raw_handle());
    }
}
