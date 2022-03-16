use crate::{
    ipc::ThreadCommandBuilder, memory::MemoryBlock, res::CtrResult, srv::get_service_handle_direct,
    utils::convert::try_usize_into_u32, Handle,
};
#[cfg(target_os = "horizon")]
use crate::{res::GenericResultCode, utils::cstring};
use core::sync::atomic::{AtomicU32, Ordering};
#[cfg(target_os = "horizon")]
use cstr_core::CString;
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive)]
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

fn httpc_initialize(
    service_handle: &Handle,
    shared_memory_block_size: usize,
    shared_memory_block_handle: &Handle,
) -> CtrResult {
    let shared_memory_block_size = try_usize_into_u32(shared_memory_block_size)?;

    let mut command = ThreadCommandBuilder::new(0x1u16);
    command.push(shared_memory_block_size);
    command.push_curent_process_id();
    command.push_shared_handles(&[shared_memory_block_handle])?;

    let mut parser = command.build().send_sync_request(service_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_initialize_connection_session(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0x8u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push_curent_process_id();

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_create_context(
    method: RequestMethod,
    url: &str,
) -> CtrResult<HttpContextHandle> {
    let c_url = cstring::parse_result(CString::new(url))?;
    let url_bytes = c_url.as_bytes_with_nul();
    let url_len = try_usize_into_u32(url_bytes.len())?;

    let mut command = ThreadCommandBuilder::new(0x2u16);
    command.push(url_len);
    command.push(method);
    command.push_read_buffer(url_bytes);

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_httpc_service_raw_handle())?;
    parser.pop_result()?;

    Ok(parser.pop().into())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_set_proxy_default(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0xEu16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_add_request_header_field(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
    header_name: &str,
    value: &str,
) -> CtrResult {
    let c_header_name = cstring::parse_result(CString::new(header_name))?;
    let header_name_bytes = c_header_name.as_bytes_with_nul();
    let header_name_len = try_usize_into_u32(header_name_bytes.len())?;

    let c_value = cstring::parse_result(CString::new(value))?;
    let value_bytes = c_value.as_bytes_with_nul();
    let value_len = try_usize_into_u32(value_bytes.len())?;

    let mut command = ThreadCommandBuilder::new(0x11u16);

    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(header_name_len);
    command.push(value_len);
    command.push_static_buffer(header_name_bytes, 3);
    command.push_read_buffer(value_bytes);

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_add_post_data_ascii(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
    post_field_name: &str,
    value: &str,
) -> CtrResult {
    if !value.is_ascii() {
        return Err(GenericResultCode::InvalidValue.into());
    }

    let c_post_field_name = cstring::parse_result(CString::new(post_field_name))?;
    let post_field_name_bytes = c_post_field_name.as_bytes_with_nul();
    let post_field_name_len = try_usize_into_u32(post_field_name_bytes.len())?;

    let c_value = cstring::parse_result(CString::new(value))?;
    let value_bytes = c_value.as_bytes_with_nul();
    let value_len = try_usize_into_u32(value_bytes.len())?;

    let mut command = ThreadCommandBuilder::new(0x12u16);

    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(post_field_name_len);
    command.push(value_len);
    command.push_static_buffer(post_field_name_bytes, 3);
    command.push_read_buffer(value_bytes);

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_set_socket_buffer_size(
    session_handle: &Handle,
    socket_buffer_size: u32,
) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0xAu16);
    command.push(socket_buffer_size);

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_receive_data_with_timeout(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
    out_buffer: &mut [u8],
    nanosecond_timeout: u64,
) -> CtrResult {
    let out_buffer_size = try_usize_into_u32(out_buffer.len())?;

    let mut command = ThreadCommandBuilder::new(0xCu16);

    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };
    command.push(out_buffer_size);
    command.push_u64(nanosecond_timeout);
    command.push_write_buffer(out_buffer);

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

#[cfg(target_os = "horizon")]
pub(crate) fn httpc_begin_request(
    session_handle: &Handle,
    context_handle: &HttpContextHandle,
) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0x9u16);
    // This is safe since we're sending it to another process, not copying it
    unsafe { command.push(context_handle.get_raw()) };

    let mut parser = command.build().send_sync_request(session_handle)?;
    parser.pop_result()?;

    Ok(())
}

pub(crate) struct HttpContextHandle(u32);

#[cfg(target_os = "horizon")]
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
        let mut command = ThreadCommandBuilder::new(0x3u16);
        command.push(self.0);
        command
            .build()
            .send_sync_request_with_raw_handle(get_httpc_service_raw_handle());
    }
}
