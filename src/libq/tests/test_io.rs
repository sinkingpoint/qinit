use std::io::Read;

/// Tests that we can give a BufferReader a buffer and it'll read it back to us
#[test]
fn test_buffer_reader_reads() {
    let src_buffer = [1, 2, 3, 4, 5, 5, 4, 3, 2, 1];
    let mut reader = libq::io::BufferReader::new(&src_buffer);
    let mut dst_buffer = [0; 10];
    match reader.read(&mut dst_buffer) {
        Ok(n) => assert_eq!(n, src_buffer.len()),
        Err(e) => assert!(false, e.to_string())
    };
    
    for (pos, e) in dst_buffer.iter().enumerate() {
        assert_eq!(e, &src_buffer[pos]);
    }
}