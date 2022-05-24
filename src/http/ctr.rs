use super::{
    get_httpc_service_raw_handle, httpc_add_post_data_ascii, httpc_add_request_header_field,
    httpc_begin_request, httpc_create_context, httpc_initialize_connection_session,
    httpc_receive_data_with_timeout, httpc_set_proxy_default, httpc_set_socket_buffer_size,
    DefaultRootCert, HttpContextHandle, RequestMethod, RequestStatus,
};
use crate::{
    ipc::Command, res::CtrResult, srv::get_service_handle_direct, utils::base64_encode, Handle,
};
use no_std_io::{EndianRead, EndianWrite};

#[derive(EndianRead, EndianWrite)]
struct AddDefaultCertIn {
    context_handle: u32,
    cert: u32,
}

#[derive(EndianRead, EndianWrite)]
struct ClientCertDefaultIn {
    context_handle: u32,
    cert: u32,
}

#[derive(EndianRead, EndianWrite)]
struct DownloadSizeStateOut {
    download_size: u32,
    content_size: u32,
}

pub struct HttpContext {
    session_handle: Handle,
    context_handle: HttpContextHandle,
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl HttpContext {
    pub fn new(url: &str, method: RequestMethod) -> CtrResult<Self> {
        let context_handle = httpc_create_context(method, url)?;
        let session_handle = get_service_handle_direct("http:C")?;

        httpc_initialize_connection_session(&session_handle, &context_handle)?;
        httpc_set_proxy_default(&session_handle, &context_handle)?;

        Ok(Self {
            session_handle,
            context_handle,
        })
    }

    pub fn add_default_cert(&self, cert: DefaultRootCert) -> CtrResult {
        let input = AddDefaultCertIn {
            context_handle: unsafe { self.context_handle.get_raw() },
            cert: cert as u32,
        };
        let raw_handle = unsafe { self.session_handle.get_raw() };
        Command::new(0x250080, input).send(raw_handle)
    }

    pub fn set_client_cert_default(&self) -> CtrResult {
        let input = ClientCertDefaultIn {
            context_handle: unsafe { self.context_handle.get_raw() },
            cert: 0x40,
        };
        let raw_handle = unsafe { self.session_handle.get_raw() };
        Command::new(0x280080, input).send(raw_handle)
    }

    pub fn add_header(&self, header_name: &str, value: &str) -> CtrResult {
        httpc_add_request_header_field(
            &self.session_handle,
            &self.context_handle,
            header_name,
            value,
        )
    }

    pub fn add_post_ascii_field(&self, post_field_name: &str, value: &str) -> CtrResult {
        httpc_add_post_data_ascii(
            &self.session_handle,
            &self.context_handle,
            post_field_name,
            value,
        )
    }

    pub fn add_post_base64_field<T: AsRef<[u8]>>(
        &self,
        post_field_name: &str,
        value: T,
    ) -> CtrResult {
        self.add_post_ascii_field(post_field_name, &base64_encode(value))
    }

    pub fn set_socket_buffer_size(&self, socket_buffer_size: u32) -> CtrResult {
        httpc_set_socket_buffer_size(&self.session_handle, socket_buffer_size)
    }

    pub fn get_download_size_state(&self) -> CtrResult<(u32, u32)> {
        let raw_context_handle = unsafe { self.context_handle.get_raw() };
        let result: DownloadSizeStateOut =
            Command::new(0x60040, raw_context_handle).send(get_httpc_service_raw_handle())?;
        Ok((result.download_size, result.content_size))
    }

    pub fn cancel_connection(&self) -> CtrResult {
        let raw_context_handle = unsafe { self.context_handle.get_raw() };
        Command::new(0x40040, raw_context_handle).send(get_httpc_service_raw_handle())
    }

    pub fn download_data_into_buffer_with_timeout(
        &self,
        out_buffer: &mut [u8],
        nanosecond_timeout: u64,
    ) -> CtrResult {
        httpc_begin_request(&self.session_handle, &self.context_handle)?;
        httpc_receive_data_with_timeout(
            &self.session_handle,
            &self.context_handle,
            out_buffer,
            nanosecond_timeout,
        )?;

        let (downloaded_size, _content_size) = self.get_download_size_state()?;

        if out_buffer.len() < (downloaded_size as usize) {
            self.cancel_connection()?;
        }

        Ok(())
    }

    pub fn download_data_into_buffer(&self, out_buffer: &mut [u8]) -> CtrResult {
        self.download_data_into_buffer_with_timeout(out_buffer, 60000000000)
    }

    pub fn get_response_status_code(&self) -> CtrResult<u32> {
        let raw_context_handle = unsafe { self.context_handle.get_raw() };
        let raw_handle = unsafe { self.session_handle.get_raw() };
        Command::new(0x220040, raw_context_handle).send(raw_handle)
    }

    pub fn get_request_status(&self) -> CtrResult<RequestStatus> {
        let raw_handle = unsafe { self.session_handle.get_raw() };
        let raw_context_handle = unsafe { self.context_handle.get_raw() };
        let status: u32 = Command::new(0x50040, raw_context_handle).send(raw_handle)?;
        Ok(status.into())
    }
}
