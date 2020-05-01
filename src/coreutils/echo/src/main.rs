fn main() {
    let mut buffer = Vec::new();
    for arg in std::env::args().skip(1) {
        buffer.extend(arg.as_bytes());
        buffer.push(b' ');
    }

    let length = buffer.len();
    if length > 0 {
        buffer[length-1] = b'\n';
    }
    else {
        buffer.push(b'\n');
    }

    libq::io::full_write_bytes(libq::io::STDOUT_FD, &buffer[..]).unwrap();
}
