use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;

mod tcp;

#[derive(Clone, Copy, Debug, Hash,Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut interface = tun_tap::Iface::without_packet_info("tun1", tun_tap::Mode::Tun)?;
    let mut buffer = [0u8; 1504];
    loop {
        let read = interface.recv(&mut buffer[..])?;
        /*
        //let flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        let proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        */
/*        if proto != 0x0800 {
            continue; //not ipv4
        }*/
        eprintln!("read {} bytes: {:x?  }", read, &buffer[..read]);

        match etherparse::Ipv4HeaderSlice::from_slice(&buffer[..read])
        {
            Ok(ip_header) => { //p is just header
                let src_address = ip_header.source_addr();
                let dest_address = ip_header.destination_addr();
                let protocol = ip_header.protocol();

                if protocol != 0x06 {
                    //not tcp
                    continue;
                }

                let ip_header_size = ip_header.slice().len();
                match etherparse::TcpHeaderSlice::from_slice(&buffer[ip_header_size..read]) {
                    Ok(tcp_header) => {
                        use std::collections::hash_map::Entry;
                        let tcp_header_size = tcp_header.slice().len();
                        let data_start = ip_header_size + tcp_header_size;

                        match connections
                            .entry(Quad{
                            src: (src_address, tcp_header.source_port()),
                            dst: (dest_address, tcp_header.destination_port())
                        }) {
                            Entry::Occupied(mut c) => {
                                c.get_mut().on_packet(&mut interface, ip_header, tcp_header, &buffer[data_start..read])?;
                            },
                            Entry::Vacant(e) => {
                                if let Some(c) = tcp::Connection::accept(
                                    &mut interface,
                                    ip_header,
                                    tcp_header,
                                    &buffer[data_start..read]
                                )? {
                                    e.insert(c);
                                }
                            }
                        }


                    }
                    Err(e) => {
                        eprintln!("ignored packet: {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("ignored packet: {:?}", e);
            }
        }
    }
}
