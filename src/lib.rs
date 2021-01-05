use std::alloc::{alloc, Layout};
use std::mem::{size_of, align_of};
use std::io::{Read, Result};
use std::ptr;
pub struct BadRingBuffer<T> {
    read: usize,
    write: usize,
    read_wrap: u8,
    write_wrap: u8,
    start_ptr: *mut T,
    capacity: usize,

}

impl<T> BadRingBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        // Define memory layout
        let layout = Layout::from_size_align(
            capacity * size_of::<T>(),
            align_of::<T>())
            .expect("could not create memory layout");

        //Allocate memory according to defined layout
        let mem = unsafe { alloc(layout) };
        
        // Cast ptr to the current T size because alloc always returns a u8
        let start_ptr = mem.cast::<T>();

        Self {
            read: 0,
            write: 0,
            read_wrap: 0,
            write_wrap: 0,
            start_ptr,
            capacity,
        }
    }

    pub fn push(&mut self, value: T) {
        unsafe {
            // Take the current writing point calculate its offset 
            // from the buffer starting mem address. And write the value to 
            // that memory address.
            let next_writabel_address = self.start_ptr.offset(self.write as isize);
            ptr::write(next_writabel_address, value);

            //Increment the writing point
            self.write += 1;
            // if self.write == 0 {
            //     self.write_wrap = self.write_wrap.wrapping_add(1);
            // }
        }
    }
}

impl<T> Iterator for BadRingBuffer<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.read_wrap == self.write_wrap && self.read == self.write {
            return None
        }

        if self.read_wrap < self.write_wrap

        let p = unsafe { self.start_ptr.offset(0).read() };

        Some(p)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_empty() {
        let mut rb = BadRingBuffer::<u8>::with_capacity(8);
        assert!(rb.next().is_none());
    }

    #[test]
    fn test_write() {
        let mut rb = BadRingBuffer::with_capacity(2);
        rb.push(0);
        rb.push(1);
        assert_eq!(rb.next(), Some(0));
        assert_eq!(rb.next(), Some(1));
    }
}