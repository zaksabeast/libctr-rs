use crate::res::{CtrResult, GenericResultCode};
use core::{convert::TryInto, marker::PhantomData};
use safe_transmute::TriviallyTransmutable;

// TriviallyTransmutable guarantees any transmuted data will be valid data.
// Copy is necessary to read unaligned data.
pub struct StaticBufferIterator<'a, T: TriviallyTransmutable + Copy + 'a> {
    phantom: PhantomData<&'a ()>,
    pointer: *const T,
    length: isize,
    offset: isize,
}

impl<'a, T: TriviallyTransmutable + Copy> StaticBufferIterator<'a, T> {
    /// # Safety
    /// The caller needs to guarantee the buffer comes from kernel translation.
    /// Behavior is undefined for all of the same conditions as [slice::from_raw_parts](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety),
    /// except the pointer may be unaligned.
    pub(super) unsafe fn new(pointer: *const T, length: usize) -> CtrResult<Self> {
        Ok(Self {
            phantom: PhantomData,
            pointer,
            length: length
                .try_into()
                .map_err(|_| GenericResultCode::InvalidValue)?,
            offset: 0,
        })
    }
}

impl<'a, T: TriviallyTransmutable + Copy> Iterator for StaticBufferIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.length {
            None
        } else {
            // This is safe since:
            // - all safety guidelines in [slice::from_raw_parts](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety) will have been followed
            // - Copy is being enforced, so all safety guidelines in [ptr::read_unaligned](https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html#safety) will also have been followed
            let item = unsafe { self.pointer.offset(self.offset).read_unaligned() };
            self.offset += 1;
            Some(item)
        }
    }
}

// TriviallyTransmutable guarantees any transmuted data will be valid data.
// Copy is necessary to read unaligned data.
pub struct StaticBufferMutIterator<'a, T: TriviallyTransmutable + Copy + 'a> {
    phantom: PhantomData<&'a ()>,
    pointer: *mut T,
    length: isize,
    offset: isize,
}

impl<'a, T: TriviallyTransmutable + Copy> StaticBufferMutIterator<'a, T> {
    /// `length` must be `usize` since a negative length doesn't make sense.
    /// However, `length` follows the same rules as slice::from_raw_parts, which limits what length can be.
    /// This is also necessary to guarantee the length can be obtained later and used as a usize.
    ///
    /// # Safety
    /// The caller needs to guarantee the buffer comes from kernel translation.
    /// Behavior is undefined for all of the same conditions as [slice::from_raw_parts](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety),
    /// except the pointer may be unaligned.
    pub(super) unsafe fn new(pointer: *mut T, length: usize) -> CtrResult<Self> {
        Ok(Self {
            phantom: PhantomData,
            pointer,
            length: length
                .try_into()
                .map_err(|_| GenericResultCode::InvalidValue)?,
            offset: 0,
        })
    }

    pub fn get_pointer(&self) -> *mut T {
        self.pointer
    }

    pub fn get_length(&self) -> usize {
        // Since `new` guarantees the length is a usize, this conversion is safe
        self.length as usize
    }
}

impl<'a, T: TriviallyTransmutable + Copy + 'a> Iterator for StaticBufferMutIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.length {
            None
        } else {
            // This is safe since:
            // - all safety guidelines in [slice::from_raw_parts](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html#safety) will have been followed
            // - Copy is being enforced, so all safety guidelines in [ptr::read_unaligned](https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html#safety) will also have been followed
            let item = unsafe { self.pointer.offset(self.offset).as_mut().unwrap() };
            self.offset += 1;
            Some(item)
        }
    }
}
