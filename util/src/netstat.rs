extern crate netstat;

use netstat::*;

pub fn tcp_port_to_pid(port: u16) -> Option<u32> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;
    
    let sockets_info = match get_sockets_info(af_flags, proto_flags) {
        Ok(si) => si,
        Err(_) => {
            return None;
        }
    };

    for si in sockets_info {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                if tcp_si.local_port == port && si.associated_pids.len() != 0 {
                    return Some(si.associated_pids[0]);
                }
            },
            ProtocolSocketInfo::Udp(_) => {

            },
        }
    }

    None
}