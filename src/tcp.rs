use std::io;
//use crate::tcp::State::Listen;

pub enum State {
    //Closed,
    //Listen,
    SynRcvd,
    Estab
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    ip: etherparse::Ipv4Header
}

/*
Send Sequence Space (RFC 793 S3.2)

    1         2          3          4
    ----------|----------|----------|----------
           SND.UNA    SND.NXT    SND.UNA+SND.WND

1 - old sequence numbers which have been acknowledged
2 - sequence numbers of unacknowledged data
3 - sequence numbers allowed for new data transmission
4 - future sequence numbers which are not yet allowed
*/

struct SendSequenceSpace {
    un_ack: u32, //una
    next: u32,
    wnd: u16,
    urgent_pointer: bool, //deprecated?
    wl1: u32, //segment sequence number used for last window update
    wl2: u32, //segment acknowledgment number used for last window update
    iss: u32,
}

/*
Receive Sequence Space (RFC 793 S3.2)

1          2          3
----------|----------|----------
       RCV.NXT    RCV.NXT+RCV.WND

1 - old sequence numbers which have been acknowledged
2 - sequence numbers allowed for new reception
3 - future sequence numbers which are not yet allowed
*/

struct RecvSequenceSpace {
    next: u32,
    wnd: u16,
    up: bool,
    irs: u32,
}

impl Connection {
    pub fn accept<'a>(
        nic: &mut tun_tap::Iface, //this is stupid, 1:23:40
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        _data: &'a [u8],
    ) -> io::Result<Option<Self>> {
        let mut buf = [0u8; 1500]; //we are ignoring zero copy

        if !tcp_header.syn() {
            return Ok(None); //SYN packets only
        }

        let iss = 0;

        let mut con = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                iss: iss,
                un_ack: iss,
                next: iss + 1,
                wnd: 10,
                urgent_pointer: false,
                wl1: iss,
                wl2: iss
            },
            recv: RecvSequenceSpace {
                irs: tcp_header.sequence_number(),
                next: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                up: false
            },
            ip: etherparse::Ipv4Header::new(
            0,
            64,
            etherparse::IpTrafficClass::Tcp,
            ip_header.destination_addr().octets(),
            ip_header.source_addr().octets(),
            )
        };

        let mut syn_ack = etherparse::TcpHeader::new(
            tcp_header.destination_port(),
            tcp_header.source_port(),
            con.send.iss, //todo: random
            con.send.wnd,
        );

        syn_ack.acknowledgment_number = con.recv.next; //what we expect next in bytes
        syn_ack.syn = true;
        syn_ack.ack = true;
        con.ip.set_payload_len(syn_ack.header_len() as usize + 0);

        //kernel does it by itself
        //syn_ack.checksum = syn_ack.calc_checksum_ipv4(&ip, &[]).expect("failed to calculate checksum");

        let unwritten = {
            let mut unwritten = &mut buf[..];
            con.ip.write(&mut unwritten);
            syn_ack.write(&mut unwritten);
            unwritten.len()
        };
        println!("length:  {:0x}", unwritten);

        eprintln!("responding with {:02x?}", &buf[..buf.len()-unwritten]);
        nic.send(&buf[..buf.len()-unwritten])?;
        Ok(Some(con))
    }

    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface, //this is stupid, 1:23:40
        ip_header: etherparse::Ipv4HeaderSlice<'a>,
        tcp_header: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<()> {
        match self.state {
            State::SynRcvd => {

            },
            State::Estab => {
                unimplemented!();
            },
        }
        Ok(())
    }
}