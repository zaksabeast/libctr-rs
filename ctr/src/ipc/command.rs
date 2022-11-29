use crate::{
    res::{error, CtrResult, ResultCode},
    svc,
};
use core::{mem, slice};
use no_std_io::{EndianRead, EndianWrite, Reader, StreamContainer, StreamWriter, Writer};

use super::{static_buffer, StaticBuffer};

const COMMAND_BUFFER_SIZE: usize = 0x100;
const STATIC_BUFFER_SIZE: usize = 0x80;

#[inline(always)]
#[ctr_macros::hos]
unsafe fn get_thread_local_storage() -> *mut u8 {
    let ret: *mut u8;
    core::arch::asm!("mrc p15, 0, {}, c13, c0, 3", out(reg) ret);
    ret
}

#[inline(always)]
fn get_thread_command_buffer() -> &'static mut [u8] {
    // This is safe because the command buffer is valid for 64 u32 reads/writes
    unsafe {
        slice::from_raw_parts_mut(get_thread_local_storage().offset(0x80), COMMAND_BUFFER_SIZE)
    }
}

#[inline(always)]
fn get_thread_static_buffers() -> &'static mut [u8] {
    // This is safe because the static buffers are valid for 24 u32 reads/writes
    unsafe {
        slice::from_raw_parts_mut(get_thread_local_storage().offset(0x180), STATIC_BUFFER_SIZE)
    }
}

#[cfg(target_os = "horizon")]
#[inline(always)]
pub(crate) fn backup_thread_command_buffer<const SIZE: usize>() -> [u8; SIZE] {
    use core::convert::TryInto;
    get_thread_command_buffer()[0..SIZE].try_into().unwrap()
}

#[cfg(target_os = "horizon")]
#[inline(always)]
pub(crate) fn restore_thread_command_buffer<const SIZE: usize>(backup: [u8; SIZE]) {
    get_thread_command_buffer().checked_write_le(0, &backup);
}

#[inline(always)]
fn get_thread_static_buffer_stream() -> StreamContainer<&'static mut [u8]> {
    StreamContainer::new(get_thread_static_buffers())
}

#[inline(always)]
fn get_static_buffer_id_index(buffer_id: u16) -> usize {
    (buffer_id as usize) * 2 * mem::size_of::<u32>()
}

#[inline(always)]
fn read_static_buffer(buffer_id: u16) -> StaticBuffer {
    let index = get_static_buffer_id_index(buffer_id);
    get_thread_static_buffers().read_le(index).unwrap()
}

#[inline(always)]
pub fn set_static_buffer(static_buffer: &StaticBuffer) {
    let index = get_static_buffer_id_index(static_buffer.id());
    get_thread_static_buffers()
        .write_le(index, static_buffer)
        .unwrap();
}

#[inline(always)]
pub(super) fn make_header(command_id: u16, normal_params: u16, translate_params: u16) -> u32 {
    ((command_id as u32) << 16)
        | (((normal_params & 0x3F) << 6) as u32)
        | ((translate_params & 0x3F) as u32)
}

#[inline(always)]
fn get_header_command_id(header: u32) -> u16 {
    (header >> 16) as u16
}

pub struct Command<T: EndianRead + EndianWrite = ()> {
    header: u32,
    data: T,
    // TODO: At some point a structure representing
    // all static buffers should be available,
    // rather than a single static buffer
    static_buffer_output: Option<StaticBuffer>,
}

impl<T: EndianRead + EndianWrite> Command<T> {
    #[inline(always)]
    pub fn new(header: u32, data: T) -> Self {
        Self {
            header,
            data,
            static_buffer_output: None,
        }
    }

    #[inline(always)]
    pub fn new_with_static_out(header: u32, data: T, static_buffer_output: StaticBuffer) -> Self {
        Self {
            header,
            data,
            static_buffer_output: Some(static_buffer_output),
        }
    }

    #[inline(always)]
    pub fn new_from_parts<CommandId: Into<u16>>(
        command_id: CommandId,
        normal_params: u16,
        translate_params: u16,
        data: T,
    ) -> Self {
        Self::new(
            make_header(command_id.into(), normal_params, translate_params),
            data,
        )
    }

    #[inline(always)]
    pub fn new_from_parts_with_static_out<CommandId: Into<u16>>(
        command_id: CommandId,
        normal_params: u16,
        translate_params: u16,
        data: T,
        static_buffer_output: StaticBuffer,
    ) -> Self {
        Self::new_with_static_out(
            make_header(command_id.into(), normal_params, translate_params),
            data,
            static_buffer_output,
        )
    }

    #[inline(always)]
    pub fn into_data(self) -> T {
        self.data
    }

    #[inline(always)]
    pub fn read() -> CtrResult<Self> {
        let command_buffer = get_thread_command_buffer();
        let header = command_buffer.read_le(0).unwrap();
        let data = command_buffer
            .read_le::<T>(4)
            .map_err(|_| error::invalid_command())?;

        Ok(Self {
            header,
            data,
            static_buffer_output: None,
        })
    }

    #[inline(always)]
    pub fn current_header() -> u32 {
        let cmd_buf = get_thread_command_buffer();
        cmd_buf.default_read_le(0)
    }

    #[inline(always)]
    pub fn current_command_id() -> u16 {
        get_header_command_id(Self::current_header())
    }

    #[inline(always)]
    pub fn send_and_get_command<Response: EndianRead + EndianWrite>(
        &self,
        raw_handle: u32,
    ) -> CtrResult<Command<Response>> {
        let mut cmd_buf = get_thread_command_buffer();

        cmd_buf.write_le(0, &self.header).unwrap();
        cmd_buf.write_le(4, &self.data)?;

        let static_buffer_backup = if let Some(static_buffer_output) = &self.static_buffer_output {
            let backup = read_static_buffer(static_buffer_output.id());
            set_static_buffer(static_buffer_output);
            Some(backup)
        } else {
            None
        };

        let sync_request_result = svc::send_raw_sync_request(raw_handle);

        if let Some(static_buffer_backup) = static_buffer_backup {
            set_static_buffer(&static_buffer_backup);
        }

        sync_request_result?;

        let header = cmd_buf.read_le(0)?;

        let result_code: ResultCode = cmd_buf.read_le(4)?;

        result_code.into_result()?;

        let res: Response = cmd_buf.read_le(8)?;

        Ok(Command::new(header, res))
    }

    #[inline(always)]
    pub fn send<Response: EndianRead + EndianWrite>(&self, raw_handle: u32) -> CtrResult<Response> {
        Ok(self.send_and_get_command(raw_handle)?.into_data())
    }

    #[inline(always)]
    pub fn write(&self) -> WrittenCommand {
        let mut cmd_buf = get_thread_command_buffer();
        cmd_buf.checked_write_le(0, &self.header);
        cmd_buf.checked_write_le(4, &self.data);

        if let Some(static_buffer_output) = &self.static_buffer_output {
            let mut static_buf = get_thread_static_buffer_stream();
            static_buf.write_stream_le(static_buffer_output).unwrap();
        }

        WrittenCommand
    }

    #[inline(always)]
    pub fn validate_buffer_id(param_number: usize, buffer_id: u16) -> CtrResult {
        let static_buffer_header: u32 = get_thread_command_buffer()
            .read_le(param_number * 4)
            .map_err(|_| error::invalid_argument())?;
        let is_valid = static_buffer_header & 0x3c0f == static_buffer::make_header(0, buffer_id);

        if is_valid {
            Ok(())
        } else {
            Err(error::invalid_command())
        }
    }

    #[inline(always)]
    pub fn validate_header<Header: Into<u32>>(header: Header) -> CtrResult {
        if Self::current_header() == header.into() {
            Ok(())
        } else {
            Err(error::invalid_command())
        }
    }
}

pub struct WrittenCommand;

impl WrittenCommand {
    #[inline(always)]
    pub fn reply_and_receive(
        &self,
        raw_handles: &[u32],
        reply_target: Option<usize>,
    ) -> (usize, ResultCode) {
        svc::reply_and_receive(raw_handles, reply_target)
    }
}
