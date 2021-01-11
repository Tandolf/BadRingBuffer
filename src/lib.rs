use std::io::{Read, Write, Result};
use std::alloc::{alloc, Layout};
use std::mem::{size_of, align_of};
use::std::ptr;

// -----------------------------------------------------------------------------
//    - BadRingBuffer struct -
// -----------------------------------------------------------------------------
pub struct BadRingBuffer<T> {
    head: usize,
    tail: usize,
    start_ptr: *mut T,
    capacity: usize,
    count: usize,
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
            head: 0,
            tail: 0,
            start_ptr,
            capacity,
            count: 0
        }
    }

    pub fn empty(&self) -> bool {
        self.count == 0
    }

    pub fn full(&self) -> bool {
        self.count == self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn push(&mut self, value: T) {
        unsafe {
            let next_writabel_address = self.start_ptr.offset(self.head as isize);
            ptr::write(next_writabel_address, value);
        }

        // if we still have room increment the count
        if self.count < self.capacity {
            self.count += 1;
        }

        // increment and wrap if needed
        self.head = (self.head + 1) % self.capacity;

        // if head has passed the tail and buffer is full ]
        // we increment the tail to read the oldest generation
        if self.head > self.tail && self.count == self.capacity {
            self.tail += 1;
        }
    }

    pub fn clear(&mut self) {
       self.count = 0;
       self.head = 0;
       self.tail = 0;
    }

    pub fn drain(&mut self) -> Vec<T> {
        let values = self.collect::<Vec<_>>();
        self.clear();
        values 
    }
}

// -----------------------------------------------------------------------------
//    - Iterator impl -
// -----------------------------------------------------------------------------
impl<T> Iterator for BadRingBuffer<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.empty() { 
            return None
        }

        let p = unsafe { self.start_ptr.offset(self.tail as isize).read() };

        // increment the tail after value read
        self.tail = (self.tail + 1) % self.capacity;

        // decrement count after read
        self.count -= 1;

        Some(p)
    }
}

// -----------------------------------------------------------------------------
//    - Read imp - 
// -----------------------------------------------------------------------------
impl Read for BadRingBuffer<u8> {
    
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut index = 0;
        let buf_len = buf.len();
        while let Some(value) = self.next() {
            buf[index] = value
            ;
            index += 1;
            if index == buf_len {
                break;
            }
        }
        Ok(index)
    }
}

// -----------------------------------------------------------------------------
//     - Write impl -
// -----------------------------------------------------------------------------
impl Write for BadRingBuffer<u8> {

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        
        buf.iter().for_each(|v| self.push(*v));

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        self.clear();
        Ok(())
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
    fn test_is_empty() {
        let rb = BadRingBuffer::<u8>::with_capacity(8);
        assert!(rb.empty());
    }

    #[test]
    fn test_read_capacity() {
        let capacity = 8;
        let rb = BadRingBuffer::<u8>::with_capacity(capacity);
        assert_eq!(rb.capacity(), capacity);
    }

    #[test]
    fn test_is_full() {
        let mut rb = BadRingBuffer::with_capacity(2);
        rb.push(1);
        rb.push(2);
        assert!(rb.full());
    }

    #[test]
    fn test_non_wrapping_write() {
        let mut rb = BadRingBuffer::with_capacity(3);
        rb.push(0);
        rb.push(1);
        rb.push(2);
        assert_eq!(rb.next(), Some(0));
        assert_eq!(rb.next(), Some(1));
        assert_eq!(rb.next(), Some(2));
    }

    #[test]
    fn test_wrapping_write() {
        let mut rb = BadRingBuffer::with_capacity(2);
        rb.push(0);
        rb.push(1);
        rb.push(2);
        assert_eq!(rb.next(), Some(1));
        assert_eq!(rb.next(), Some(2));
    }

    #[test]
    fn test_read() {
        let mut buf = [0; 1024];
        let mut rb = BadRingBuffer::with_capacity(4);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4);

        let read_bytes = rb.read(&mut buf).unwrap();

        assert_eq!(&buf[0..read_bytes], &[1,2,3,4])
    }

    #[test]
    fn test_should_drain() {
        let mut rb = BadRingBuffer::with_capacity(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);

        let values = rb.drain();
        assert_eq!(values, &[1,2,3]);
        assert!(rb.empty());
    }

    #[test]
    fn test_write() {
        let buf = [1, 2, 3, 4];
        let mut rb = BadRingBuffer::with_capacity(4);

        let bytes_written = rb.write(&buf).unwrap();
        assert_eq!(4, bytes_written);
        assert_eq!(rb.drain(), vec![1,2,3,4]);
    }

    #[test]
    fn test_clearing() {
        let mut rb = BadRingBuffer::with_capacity(4);
        rb.push(1);
        rb.clear();

        assert!(rb.next().is_none());
    }
}