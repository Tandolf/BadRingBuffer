## BadRingBuffer

Dont use it... Im serious... Just don't.

### Implementation
This is an `unsafe` ring buffer implemented in Rust with memory layout and pointers. When the buffer is full it will push the tail hence overwriting the oldest generation.

### TODO
- Impl option to not overwrite oldest gen and instead return an error during write time.