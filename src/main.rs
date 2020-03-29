use std::io;

fn main() -> io::Result<()> {
    let interface = tun_tap::Iface::new("tun1", tun_tap::Mode::Tun)?;
    let mut buffer = [0u8; 1504];
    let read = interface.recv(&mut buffer[..])?;
    eprintln!("read {} bytes: {:x?}", read, &buffer[..read]);
    Ok(())
}
