use crate::{
    res::{parse_result, CtrResult, GenericResultCode, ResultCode},
    safe_transmute::transmute_one_pedantic,
    svc,
    utils::convert::{try_usize_into_u32, u8_slice_to_u32},
    Handle,
};
use alloc::vec::Vec;
#[cfg(not(target_os = "horizon"))]
use core::fmt;
#[cfg(target_os = "horizon")]
use core::slice;
use core::{marker::PhantomData, mem, panic};
use safe_transmute::{transmute_one_to_bytes, transmute_to_bytes, TriviallyTransmutable};

const COMMAND_BUFFER_SIZE: usize = 64;
const STATIC_BUFFER_SIZE: usize = 32;

#[inline]
fn make_header(command_id: u16, normal_params: u32, translate_params: u32) -> u32 {
    ((command_id as u32) << 16) | ((normal_params & 0x3F) << 6) | (translate_params & 0x3F)
}

#[inline]
fn make_static_buffer_header(size: usize, buffer_id: u16) -> u32 {
    ((size as u32) << 14) | (((buffer_id as u32) & 0xF) << 10) | 0x2
}

#[inline]
fn get_static_buffer_size(header: u32) -> usize {
    (header >> 14) as usize
}

#[inline]
fn get_header_command_id(header: u32) -> u16 {
    (header >> 16) as u16
}

pub enum BufferRights {
    Read = 2,
    Write = 4,
    ReadWrite = 6,
}

#[inline]
pub fn make_buffer_header(size: usize, rights: BufferRights) -> u32 {
    ((size as u32) << 4) | 0x8 | rights as u32
}

#[inline]
fn get_buffer_size(header: u32) -> usize {
    (header >> 4) as usize
}

#[inline]
fn check_buffer_permissions(header: u32, rights: BufferRights) -> bool {
    (header & rights as u32) != 0
}

#[inline]
pub fn get_cur_process_id_header() -> u32 {
    0x20
}

#[inline]
pub fn make_shared_handles_header(number: u32) -> u32 {
    (number - 1) << 26
}

#[cfg(target_os = "horizon")]
#[inline]
unsafe fn get_thread_local_storage() -> *mut u32 {
    let ret: *mut u32;

    asm!("mrc p15, 0, {}, c13, c0, 3", out(reg) ret);
    ret
}

#[cfg(target_os = "horizon")]
#[inline]
pub(in crate) fn get_thread_command_buffer() -> &'static mut [u32] {
    // This is safe because the command buffer is valid for 64 u32 reads/writes
    unsafe {
        slice::from_raw_parts_mut(get_thread_local_storage().offset(0x20), COMMAND_BUFFER_SIZE)
    }
}

#[cfg(target_os = "horizon")]
#[inline]
fn get_thread_static_buffers() -> &'static mut [u32] {
    // This is safe because the static buffers are valid for 24 u32 reads/writes
    unsafe {
        slice::from_raw_parts_mut(get_thread_local_storage().offset(0x60), STATIC_BUFFER_SIZE)
    }
}

#[cfg(target_os = "horizon")]
struct ThreadBuffers {
    command_buffer: &'static mut [u32],
    static_buffers: &'static mut [u32],
}

#[cfg(target_os = "horizon")]
#[inline]
fn get_thread_buffers() -> ThreadBuffers {
    unsafe {
        let thread_local_storage = get_thread_local_storage();

        ThreadBuffers {
            // This is safe because the command buffer is valid for 64 u32 reads/writes
            command_buffer: slice::from_raw_parts_mut(
                thread_local_storage.offset(0x20),
                COMMAND_BUFFER_SIZE,
            ),
            // This is safe because the static buffers are valid for 24 u32 reads/writes
            static_buffers: slice::from_raw_parts_mut(
                thread_local_storage.offset(0x60),
                STATIC_BUFFER_SIZE,
            ),
        }
    }
}

#[cfg(target_os = "horizon")]
#[inline]
/// Sets static buffers 0, 1, and 2 with the provided data
pub fn set_static_buffers(data: &[u8]) {
    let static_buf = get_thread_static_buffers();

    static_buf[0] = make_static_buffer_header(data.len(), 0);
    static_buf[1] = data.as_ptr() as u32;
    static_buf[2] = make_static_buffer_header(data.len(), 1);
    static_buf[3] = data.as_ptr() as u32;
    static_buf[4] = make_static_buffer_header(data.len(), 2);
    static_buf[5] = data.as_ptr() as u32;
}

#[cfg(not(target_os = "horizon"))]
#[inline]
pub fn set_static_buffers(_data: &[u8]) {}

/// An abstraction of the thread command buffer loosely based on Citra's [RequestBuilder](https://github.com/citra-emu/citra/blob/1c8461fdea1106c68729643525c365afc93f5621/src/core/hle/ipc_helpers.h#L46).
///
/// This provides a few benefits:
/// - A safe interface for the thread command buffer, which is always non-null and aligned
/// - A more idiomatic way to write commands
/// - An interface that can be mocked for unit testing on systems other than the 3ds
///
/// This interface aims to be inlined when possible and write to the thread command buffer directly to reduce overhead.
///
/// ## Warning
///
/// Because this provides an interface to talk with the single thread command buffer and writes to the thread command buffer as commands are being built,
/// two instances of ThreadCommandBuilder should not be used at the same time.  This interface is memory safe, but can be used logically wrong.
///
/// For example, the following is invalid:
/// ```
/// use libctr_rs::ipc::ThreadCommandBuilder;
/// let mut command_1 = ThreadCommandBuilder::new(1u16);
/// let mut command_2 = ThreadCommandBuilder::new(2u16);
///
/// command_2.push(0u32);
/// command_1.push(1u32);
/// ```
/// The end result will have the header for command_2 and the parameter for command_1.
///
/// Similarly, pushing parameters with ThreadCommandBuilder and simultaneously reading parameters with ThreadCommandParser will cause issues.
///
/// For example:
/// ```
/// use libctr_rs::ipc::{ThreadCommandBuilder, ThreadCommandParser};
/// let command = ThreadCommandBuilder::new(1u16);
/// let parser = ThreadCommandParser::new();
/// ```
/// The parser will read the command that was just written, not a previous command.
///
/// This could be resolved by copying from an internal buffer, but it has been deemed a worthy trade-off given:
/// - The additional overhead for frequently used, low level operation
/// - The natural way the thread command buffer is intended to be used
/// - There's no benefit building two commands simultaneously since only one command can be issued at a time per thread anyways
///
/// Instead, this should be used in one of the following ways:
/// - In a wrapper that abstracts away the thread command buffer
/// - In a service handler where some form of svc::reply_and_receive will follow
pub struct ThreadCommandBuilder<'a> {
    phantom: PhantomData<&'a ()>,
    param_count: usize,
    static_buffer_count: usize,
    normal_param_count: usize,
    translate_param_count: usize,
    command_id: u16,

    /// Where normal params are stored
    #[cfg(target_os = "horizon")]
    param_pool: &'static mut [u32],
    /// Where static buffer params are stored
    #[cfg(target_os = "horizon")]
    static_buffer_pool: &'static mut [u32],

    /// Where normal params are stored
    #[cfg(not(target_os = "horizon"))]
    pub param_pool: [u32; COMMAND_BUFFER_SIZE],
    /// Where static buffer params are stored
    #[cfg(not(target_os = "horizon"))]
    pub static_buffer_pool: [u32; STATIC_BUFFER_SIZE],
    /// A mock buffer to store values.  Slices of this end up in the static_buffer_pool.
    #[cfg(not(target_os = "horizon"))]
    pub static_buffer: [Vec<u8>; 16],
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl<'a> ThreadCommandBuilder<'a> {
    #[cfg(target_os = "horizon")]
    /// Writes the command header into the thread command buffer
    pub fn new<T: Into<u16>>(command_id: T) -> Self {
        let thread_buffers = get_thread_buffers();

        Self {
            phantom: PhantomData,
            // A single parameter is always assumed for the header
            param_count: 1,
            static_buffer_count: 0,
            command_id: command_id.into(),
            normal_param_count: 0,
            translate_param_count: 0,
            param_pool: thread_buffers.command_buffer,
            static_buffer_pool: thread_buffers.static_buffers,
        }
    }

    #[cfg(not(target_os = "horizon"))]
    pub fn new<T: Into<u16>>(command_id: T) -> Self {
        Self {
            phantom: PhantomData,
            // A single parameter is always assumed for the header
            param_count: 1,
            static_buffer_count: 0,
            normal_param_count: 0,
            translate_param_count: 0,
            command_id: command_id.into(),
            param_pool: [0; COMMAND_BUFFER_SIZE],
            static_buffer_pool: [0; STATIC_BUFFER_SIZE],
            // Vec<u8> doesn't implement Copy, so we can't use [Vec::new(); 16]
            static_buffer: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
        }
    }

    fn inner_push_static_output_buffer<T: Into<u32>>(&mut self, param: T) {
        self.static_buffer_pool[self.static_buffer_count] = param.into();
        self.static_buffer_count += 1;
    }

    #[cfg(target_pointer_width = "32")]
    fn inner_push_static_output_buffer_usize(&mut self, param: usize) {
        self.inner_push_static_output_buffer(param as u32);
    }

    #[cfg(target_pointer_width = "64")]
    fn inner_push_static_output_buffer_usize(&mut self, param: usize) {
        self.inner_push_static_output_buffer(param as u32);
        self.inner_push_static_output_buffer((param >> 32) as u32);
    }

    /// Pushes a parameter onto the thread command buffer.
    /// This does not increment the normal or translate count
    /// and is only for use internal to this struct.
    fn inner_push<T: Into<u32>>(&mut self, param: T) {
        self.param_pool[self.param_count] = param.into();
        self.param_count += 1;
    }

    #[cfg(target_pointer_width = "32")]
    fn inner_push_usize(&mut self, param: usize) {
        self.inner_push(param as u32);
    }

    #[cfg(target_pointer_width = "64")]
    fn inner_push_usize(&mut self, param: usize) {
        self.inner_push(param as u32);
        self.inner_push((param >> 32) as u32);
    }

    /// Pushes a parameter onto the thread command buffer.
    /// Panics if used after a translate param has been pushed.
    pub fn push<T: Into<u32>>(&mut self, param: T) {
        if self.translate_param_count > 0 {
            panic!();
        }

        self.inner_push(param);
        self.normal_param_count += 1;
    }

    /// Pushes a parameter onto the thread command buffer
    pub fn push_u64(&mut self, param: u64) {
        self.push(param as u32);
        self.push((param >> 32) as u32);
    }

    pub fn push_struct<T: TriviallyTransmutable>(&mut self, data: &T) {
        transmute_one_to_bytes(data).chunks(4).for_each(|bytes| {
            self.push(u8_slice_to_u32(bytes));
        });
    }

    // Pushes the static buffer header and pointer as parameters onto the thread command buffer
    pub fn push_static_buffer<T>(&mut self, data: &'a [T], buffer_id: u16) {
        self.inner_push(make_static_buffer_header(
            data.len() * core::mem::size_of::<T>(),
            buffer_id,
        ));
        self.inner_push_usize(data.as_ptr() as usize);

        self.translate_param_count += 2;
    }

    // Pushes the static buffer header and pointer as outputs.
    pub fn push_output_static_buffer<T: TriviallyTransmutable>(
        &mut self,
        data: &'a mut [T],
        buffer_id: u16,
    ) {
        self.inner_push_static_output_buffer(make_static_buffer_header(
            data.len() * core::mem::size_of::<T>(),
            buffer_id,
        ));
        self.inner_push_static_output_buffer_usize(data.as_mut_ptr() as usize);
    }

    pub fn push_read_buffer<T>(&mut self, data: &'a [T]) {
        self.inner_push(make_buffer_header(
            data.len() * core::mem::size_of::<T>(),
            BufferRights::Read,
        ));
        self.inner_push_usize(data.as_ptr() as usize);

        self.translate_param_count += 2;
    }

    pub fn push_write_buffer<T>(&mut self, data: &'a mut [T]) {
        // This is safe because push_raw_write_buffer needs to have the same guarantees as a slice,
        // and we're using a slice as the arguments
        unsafe { self.push_raw_write_buffer(data.as_mut_ptr(), data.len()) };
    }

    /// # Safety
    /// Behavior is undefined for all of the same conditions as [slice::from_raw_parts](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety),
    pub unsafe fn push_raw_write_buffer<T>(&mut self, ptr: *mut T, size: usize) {
        self.inner_push(make_buffer_header(
            size * core::mem::size_of::<T>(),
            BufferRights::Write,
        ));
        self.inner_push_usize(ptr as usize);

        self.translate_param_count += 2;
    }

    pub fn push_raw_handle(&mut self, raw_handle: u32) {
        self.inner_push(0u32);
        self.inner_push(raw_handle);

        self.translate_param_count += 2;
    }

    pub fn push_curent_process_id(&mut self) {
        self.inner_push(get_cur_process_id_header());
        self.inner_push(0u32);

        self.translate_param_count += 2;
    }

    pub fn push_shared_handles(&mut self, handles: &[&Handle]) -> Result<(), GenericResultCode> {
        let handles_len: u32 = try_usize_into_u32(handles.len())?;

        self.inner_push(make_shared_handles_header(handles_len));

        for handle in handles.iter() {
            // This is safe because we're sending handles to another process, not duplicating them
            unsafe { self.inner_push(handle.get_raw()) };
        }

        self.translate_param_count += 1 + handles.len();

        Ok(())
    }

    fn write_header(&mut self) {
        self.param_pool[0] = make_header(
            self.command_id,
            self.normal_param_count as u32,
            self.translate_param_count as u32,
        );
    }

    pub fn build(self) -> ThreadCommand<'a> {
        self.into()
    }
}

/// This represents a built command.
/// The main difference between this and the command builder is the lack of fields this has.
/// This prevents data from being stored unecessarilly (e.g. storing the param count in a specific field).
pub struct ThreadCommand<'a> {
    phantom: PhantomData<&'a ()>,

    #[cfg(target_os = "horizon")]
    param_pool: &'static mut [u32],

    #[cfg(not(target_os = "horizon"))]
    pub param_pool: [u32; COMMAND_BUFFER_SIZE],
    #[cfg(not(target_os = "horizon"))]
    pub static_buffer: [Vec<u8>; 16],
}

impl<'a> ThreadCommand<'a> {
    #[cfg(not(target_os = "horizon"))]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            param_pool: [0; COMMAND_BUFFER_SIZE],
            static_buffer: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
        }
    }

    #[cfg(target_os = "horizon")]
    pub fn new() -> Self {
        let thread_buffers = get_thread_buffers();

        Self {
            phantom: PhantomData,
            param_pool: thread_buffers.command_buffer,
        }
    }

    pub fn reply_and_receive(
        self,
        raw_handles: &[u32],
        reply_target: Option<usize>,
    ) -> (usize, ResultCode) {
        svc::reply_and_receive(raw_handles, reply_target)
    }

    pub fn send_sync_request_with_raw_handle(
        self,
        raw_handle: u32,
    ) -> CtrResult<ThreadCommandParser> {
        svc::send_raw_sync_request(raw_handle)?;

        Ok(self.into())
    }

    pub fn send_sync_request(self, handle: &Handle) -> CtrResult<ThreadCommandParser> {
        // This is safe since we're sending it to another process, not copying it
        let raw_handle = unsafe { handle.get_raw() };
        self.send_sync_request_with_raw_handle(raw_handle)
    }
}

impl<'a> Default for ThreadCommand<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "horizon"))]
impl<'a> fmt::Debug for ThreadCommand<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ThreadCommand").finish()
    }
}

#[cfg(target_os = "horizon")]
impl<'a> From<ThreadCommandBuilder<'a>> for ThreadCommand<'a> {
    fn from(mut builder: ThreadCommandBuilder) -> Self {
        builder.write_header();

        Self {
            phantom: PhantomData,
            param_pool: builder.param_pool,
        }
    }
}

#[cfg(not(target_os = "horizon"))]
impl<'a> From<ThreadCommandBuilder<'a>> for ThreadCommand<'a> {
    fn from(mut builder: ThreadCommandBuilder) -> Self {
        builder.write_header();

        Self {
            phantom: PhantomData,
            param_pool: builder.param_pool,
            static_buffer: builder.static_buffer,
        }
    }
}

/// An abstraction of the thread command buffer loosely based on Citra's [RequestParser](https://github.com/citra-emu/citra/blob/1c8461fdea1106c68729643525c365afc93f5621/src/core/hle/ipc_helpers.h#L211).
///
/// This provides a few benefits:
/// - A safe interface to read from the thread command buffer, which is always non-null and aligned
/// - A more idiomatic way to read commands
/// - An interface that can be mocked for unit testing on systems other than the 3ds
///
/// This interface aims to be inlined when possible and write to the thread command buffer directly to reduce overhead.
///
/// ## Warning
///
/// Because this provides an interface to talk with the single thread command buffer, an instance of ThreadCommandBuilder and ThreadCommandParser will conflict with each other if used at the same time.
///
/// For example:
/// ```
/// use libctr_rs::ipc::{ThreadCommandBuilder, ThreadCommandParser};
/// let command = ThreadCommandBuilder::new(1u16);
/// let parser = ThreadCommandParser::new();
/// ```
/// The parser will read the command that was just written, not a previous command.
///
/// If you need to read parameters from a request, do it before building the response.
pub struct ThreadCommandParser {
    param_count: usize,

    #[cfg(target_os = "horizon")]
    param_pool: &'static mut [u32],

    #[cfg(not(target_os = "horizon"))]
    pub param_pool: [u32; COMMAND_BUFFER_SIZE],
}

impl ThreadCommandParser {
    #[cfg(target_os = "horizon")]
    pub fn new() -> Self {
        Self {
            // A single parameter is always assumed for the header
            param_count: 1,
            param_pool: get_thread_command_buffer(),
        }
    }

    #[cfg(not(target_os = "horizon"))]
    pub fn new() -> Self {
        Self {
            // A single parameter is always assumed for the header
            param_count: 1,
            param_pool: [0; COMMAND_BUFFER_SIZE],
        }
    }

    /// Reads the command header
    pub fn get_header(&self) -> u32 {
        self.param_pool[0]
    }

    /// Reads the command Id
    pub fn get_command_id(&self) -> u16 {
        get_header_command_id(self.get_header())
    }

    pub fn validate_header<T: Into<u16>>(
        &self,
        command_id: T,
        normal_params: u32,
        translate_params: u32,
    ) -> CtrResult<()> {
        let is_valid =
            self.get_header() == make_header(command_id.into(), normal_params, translate_params);

        if is_valid {
            Ok(())
        } else {
            Err(GenericResultCode::InvalidCommand.into())
        }
    }

    pub fn validate_buffer_id(&self, param_number: usize, buffer_id: u16) -> CtrResult<()> {
        let is_valid =
            self.param_pool[param_number] & 0x3c0f == make_static_buffer_header(0, buffer_id);

        if is_valid {
            Ok(())
        } else {
            Err(GenericResultCode::InvalidCommand.into())
        }
    }

    #[cfg(not(target_os = "horizon"))]
    pub fn pop_and_validate_buffer(
        &mut self,
        size: usize,
        rights: BufferRights,
        pointer: usize,
    ) -> CtrResult<()> {
        let header = self.pop();

        if header & 8 == 0 {
            return Err(GenericResultCode::InvalidValue.into());
        }

        if size != get_buffer_size(header) as usize {
            return Err(GenericResultCode::InvalidSize.into());
        }

        if !check_buffer_permissions(header, rights) {
            return Err(GenericResultCode::InvalidBufferRights.into());
        }

        if pointer != self.pop_usize() {
            return Err(GenericResultCode::InvalidPointer.into());
        }

        Ok(())
    }

    pub fn pop_and_validate_process_id(&mut self) -> CtrResult<u32> {
        if self.pop() == get_cur_process_id_header() {
            return Ok(self.pop());
        }

        Err(GenericResultCode::InvalidValue.into())
    }

    pub fn pop_handle(&mut self) -> CtrResult<Handle> {
        if self.pop() == 0 {
            return Ok(self.pop().into());
        }

        Err(GenericResultCode::InvalidValue.into())
    }

    /// Reads the next parameter from the thread command buffer
    pub fn pop(&mut self) -> u32 {
        let result = self.param_pool[self.param_count];
        self.param_count += 1;

        result
    }

    pub fn pop_u64(&mut self) -> u64 {
        (self.pop() as u64) | ((self.pop() as u64) << 32)
    }

    #[cfg(target_pointer_width = "32")]
    pub fn pop_usize(&mut self) -> usize {
        self.pop() as usize
    }

    #[cfg(target_pointer_width = "64")]
    pub fn pop_usize(&mut self) -> usize {
        self.pop_u64() as usize
    }

    pub fn pop_i32(&mut self) -> i32 {
        // This is safe because the value truly should be interpretted as an i32
        unsafe { mem::transmute::<u32, i32>(self.pop()) }
    }

    /// A convenient method to pop a CtrResult from the thread command buffer.
    pub fn pop_result(&mut self) -> CtrResult<ResultCode> {
        parse_result(self.pop_i32())
    }

    /// Pop trivally transmutable structs from the command buffer.
    pub fn pop_struct<T: TriviallyTransmutable>(&mut self) -> CtrResult<T> {
        let struct_size = core::mem::size_of::<T>();

        let is_unaligned = struct_size % core::mem::size_of::<u32>() != 0;
        let next_param_count = self.param_count + (struct_size / 4) + (is_unaligned as usize);

        if next_param_count > COMMAND_BUFFER_SIZE {
            return Err(GenericResultCode::InvalidSize.into());
        }

        let struct_bytes = transmute_to_bytes(&self.param_pool[self.param_count..next_param_count]);
        let popped_struct = transmute_one_pedantic::<T>(&struct_bytes[..struct_size])?;

        self.param_count = next_param_count;

        Ok(popped_struct)
    }

    /// # Safety
    /// The caller must ensure the buffer comes from kernel translation to guarantee the data is valid.
    ///
    /// For example, creating a thread command, pushing random values, then
    /// popping as a buffer will result in undefined behavior.
    ///
    /// Since the data implements TriviallyTransmutable, bad data should not
    /// affect the result of this method.
    pub unsafe fn pop_static_buffer<'a, T: 'a + TriviallyTransmutable + Copy>(
        &mut self,
    ) -> CtrResult<Vec<T>> {
        let header = self.pop();

        let ptr = self.pop_usize() as *const T;

        if ptr.is_null() {
            return Err(GenericResultCode::NulError.into());
        }

        let size = get_static_buffer_size(header);
        let length = size / core::mem::size_of::<T>();

        // https://www.3dbrew.org/wiki/IPC#Static_Buffer_Translation
        // The pointer has been translated for us by the kernel, so the pointer and size should be valid,
        // and we've made sure the pointer isn't null, just to be safe.
        let slice = core::slice::from_raw_parts::<'a, T>(ptr, length);
        Ok(slice.to_vec())
    }

    /// # Safety
    /// The caller must ensure the buffer comes from kernel translation to guarantee the data is valid.
    ///
    /// For example, creating a thread command, pushing random values, then
    /// popping as a buffer will result in undefined behavior.
    ///
    /// Since the data implements TriviallyTransmutable, bad data should not
    /// affect the result of this method.
    pub unsafe fn pop_mut_buffer<'a, T: TriviallyTransmutable + Copy>(
        &mut self,
    ) -> CtrResult<&'a mut [T]> {
        let header = self.pop();
        if !check_buffer_permissions(header, BufferRights::Write) {
            return Err(GenericResultCode::InvalidBufferRights.into());
        }

        let ptr = self.pop_usize() as *mut T;

        if ptr.is_null() {
            return Err(GenericResultCode::NulError.into());
        }

        let size = get_buffer_size(header);
        let length = size / core::mem::size_of::<T>();

        // https://www.3dbrew.org/wiki/IPC#Static_Buffer_Translation
        // The pointer has been translated for us by the kernel, so the pointer and size should be valid,
        // and we've made sure the pointer isn't null, just to be safe.
        let slice = core::slice::from_raw_parts_mut::<'a, T>(ptr, length);
        Ok(slice)
    }

    /// # Safety
    /// The caller must ensure the buffer comes from kernel translation to guarantee the data is valid.
    ///
    /// For example, creating a thread command, pushing random values, then
    /// popping as a buffer will result in undefined behavior.
    ///
    /// Since the data implements TriviallyTransmutable, bad data should not
    /// affect the result of this method.
    pub unsafe fn pop_buffer<'a, T: TriviallyTransmutable + Copy>(&mut self) -> CtrResult<&'a [T]> {
        let header = self.pop();
        if !check_buffer_permissions(header, BufferRights::Read) {
            return Err(GenericResultCode::InvalidBufferRights.into());
        }

        let ptr = self.pop_usize() as *const T;

        if ptr.is_null() {
            return Err(GenericResultCode::NulError.into());
        }

        let size = get_buffer_size(header);
        let length = size / core::mem::size_of::<T>();

        // https://www.3dbrew.org/wiki/IPC#Static_Buffer_Translation
        // The pointer has been translated for us by the kernel, so the pointer and size should be valid,
        // and we've made sure the pointer isn't null, just to be safe.
        let slice = core::slice::from_raw_parts::<'a, T>(ptr, length);
        Ok(slice)
    }
}

impl Default for ThreadCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<ThreadCommand<'a>> for ThreadCommandParser {
    fn from(command: ThreadCommand) -> Self {
        Self {
            // A single parameter is always assumed for the header
            param_count: 1,
            param_pool: command.param_pool,
        }
    }
}
