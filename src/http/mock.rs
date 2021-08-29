use super::{DefaultRootCert, RequestMethod, RequestStatus};
use crate::{res::CtrResult, utils::base64_encode};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::cell::RefCell;
use hashbrown::HashMap;

pub struct InternalHttpContext {
    pub url: String,
    pub method: RequestMethod,
    pub certs: Vec<DefaultRootCert>,
    pub has_client_cert: bool,
    // Using strings here isn't amazing, but this is for testing/mocking only
    // so it prevents the real implementation from needing lifetimes.
    pub headers: HashMap<String, String>,
    pub post_body_fields: HashMap<String, String>,
    pub socket_buffer_size: u32,
}

pub struct HttpContext {
    pub mock: RefCell<InternalHttpContext>,
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl HttpContext {
    pub fn new(url: &str, method: RequestMethod) -> CtrResult<Self> {
        let context = InternalHttpContext {
            url: url.to_string(),
            method,
            certs: vec![],
            has_client_cert: false,
            headers: HashMap::new(),
            post_body_fields: HashMap::new(),
            socket_buffer_size: 0,
        };
        Ok(Self {
            mock: RefCell::new(context),
        })
    }

    pub fn add_default_cert(&self, cert: DefaultRootCert) -> CtrResult<()> {
        self.mock.borrow_mut().certs.push(cert);
        Ok(())
    }

    pub fn set_client_cert_default(&self) -> CtrResult<()> {
        self.mock.borrow_mut().has_client_cert = true;
        Ok(())
    }

    pub fn add_header(&self, header_name: &str, value: &str) -> CtrResult<()> {
        self.mock
            .borrow_mut()
            .headers
            .insert(header_name.to_string(), value.to_string());
        Ok(())
    }

    pub fn add_post_ascii_field(&self, post_field_name: &str, value: &str) -> CtrResult<()> {
        self.mock
            .borrow_mut()
            .post_body_fields
            .insert(post_field_name.to_string(), value.to_string());
        Ok(())
    }

    pub fn add_post_base64_field<T: AsRef<[u8]>>(
        &self,
        post_field_name: &str,
        value: T,
    ) -> CtrResult<()> {
        self.add_post_ascii_field(post_field_name, &base64_encode(value))
    }

    pub fn set_socket_buffer_size(&self, socket_buffer_size: u32) -> CtrResult<()> {
        self.mock.borrow_mut().socket_buffer_size = socket_buffer_size;
        Ok(())
    }

    pub fn get_download_size_state(&self) -> CtrResult<(u32, u32)> {
        // Downloaded size, Content size
        Ok((0, 0))
    }

    pub fn cancel_connection(&self) -> CtrResult<()> {
        Ok(())
    }

    pub fn download_data_into_buffer_with_timeout(
        &self,
        out_buffer: &mut [u8],
        nanosecond_timeout: u64,
    ) -> CtrResult<()> {
        Ok(())
    }

    pub fn download_data_into_buffer(&self, out_buffer: &mut [u8]) -> CtrResult<()> {
        self.download_data_into_buffer_with_timeout(out_buffer, 60000000000)
    }

    pub fn get_response_status_code(&self) -> CtrResult<u32> {
        Ok(200)
    }

    pub fn get_request_status(&self) -> CtrResult<RequestStatus> {
        Ok(RequestStatus::RequestInProgress)
    }
}
