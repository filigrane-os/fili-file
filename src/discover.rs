// Servers announced on the local network, found the way macOS and Linux file managers find them: multicast DNS service
// discovery (DNS-SD). One UDP question per service type is sent to 224.0.0.251:5353 and the answers are collected for a
// moment: no daemon (avahi is not needed) and no dependency. NAS boxes, Macs and Linux machines running Samba or SSH
// announce `_smb._tcp` and `_sftp-ssh._tcp`; Windows PCs usually do not, they stay reachable through Connect to Server.
//
// The socket is the standard mDNS one: port 5353 (shared with other mDNS programs through SO_REUSEPORT) and a member
// of the multicast group, so the answers, which are multicast, reach us. A firewall usually lets those through and drops
// unicast answers to an ordinary port, which is why that simpler way (kept as a fallback) did not work on many networks.
// A responder that answers with the instance name alone is asked again for its address.
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::Mutex,
    time::{Duration, Instant},
};

const SERVICES: [(&str, &str); 2] = [("_smb._tcp.local", "smb"), ("_sftp-ssh._tcp.local", "sftp")];

// A server found on the network: readable name and the address to connect to (smb://192.168.1.20/)
#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    pub name: String,
    pub uri: String,
}

const PTR: u16 = 12;
const SRV: u16 = 33;
const A: u16 = 1;

#[derive(Debug, Clone, PartialEq)]
enum Data {
    Ptr(String),
    Srv(String),
    A(Ipv4Addr),
    Other,
}

#[derive(Debug, Clone, PartialEq)]
struct Record {
    name: String,
    data: Data,
}

fn be16(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*b.get(i)?, *b.get(i + 1)?]))
}

// A (possibly compressed) domain name at `pos`: the name and the position after it in the stream
fn read_name(buf: &[u8], pos: usize) -> Option<(String, usize)> {
    let (mut labels, mut p, mut end, mut jumps) = (Vec::new(), pos, None, 0);
    loop {
        let len = *buf.get(p)? as usize;
        if len == 0 {
            p += 1;
            break;
        }
        if len & 0xC0 == 0xC0 {
            let target = ((len & 0x3F) << 8) | *buf.get(p + 1)? as usize;
            end.get_or_insert(p + 2);
            jumps += 1;
            if jumps > 16 {
                return None; // pointer loop
            }
            p = target;
            continue;
        }
        labels.push(String::from_utf8_lossy(buf.get(p + 1..p + 1 + len)?).into_owned());
        p += 1 + len;
    }
    Some((labels.join("."), end.unwrap_or(p)))
}

// Every answer, authority and additional record of a DNS packet that we care about
fn parse(buf: &[u8]) -> Vec<Record> {
    let mut out = Vec::new();
    let (Some(qd), Some(an), Some(ns), Some(ar)) = (be16(buf, 4), be16(buf, 6), be16(buf, 8), be16(buf, 10)) else { return out };
    let mut pos = 12;
    for _ in 0..qd {
        let Some((_, p)) = read_name(buf, pos) else { return out };
        pos = p + 4;
    }
    for _ in 0..(an as usize + ns as usize + ar as usize) {
        let Some((name, p)) = read_name(buf, pos) else { return out };
        let (Some(rtype), Some(rdlen)) = (be16(buf, p), be16(buf, p + 8)) else { return out };
        let rdata = p + 10;
        let data = match rtype {
            PTR => read_name(buf, rdata).map(|(n, _)| Data::Ptr(n)).unwrap_or(Data::Other),
            SRV => read_name(buf, rdata + 6).map(|(n, _)| Data::Srv(n)).unwrap_or(Data::Other),
            A if rdlen == 4 => match buf.get(rdata..rdata + 4) {
                Some(b) => Data::A(Ipv4Addr::new(b[0], b[1], b[2], b[3])),
                None => Data::Other,
            },
            _ => Data::Other,
        };
        out.push(Record { name: name.to_lowercase(), data });
        pos = rdata + rdlen as usize;
    }
    out
}

// A DNS question packet (class IN, ordinary query)
fn question(items: &[(&str, u16)]) -> Vec<u8> {
    let mut p = vec![0, 0, 0, 0];
    p.extend((items.len() as u16).to_be_bytes());
    p.extend([0, 0, 0, 0, 0, 0]);
    for (name, qtype) in items {
        for label in name.split('.') {
            p.push(label.len() as u8);
            p.extend(label.as_bytes());
        }
        p.push(0);
        p.extend(qtype.to_be_bytes());
        p.extend(1u16.to_be_bytes());
    }
    p
}

// Servers described by the records: a service instance (PTR), its host (SRV) and the address of that host (A)
fn assemble(records: &[Record]) -> Vec<Server> {
    let mut servers: Vec<Server> = Vec::new();
    for r in records {
        let Data::Ptr(instance) = &r.data else { continue };
        let Some((service, scheme)) = SERVICES.iter().find(|(s, _)| r.name == *s) else { continue };
        let lower = instance.to_lowercase();
        let host = records.iter().find_map(|x| match &x.data {
            Data::Srv(t) if x.name == lower => Some(t.to_lowercase()),
            _ => None,
        });
        let Some(host) = host else { continue };
        let address = records.iter().find_map(|x| match x.data {
            Data::A(ip) if x.name == host => Some(ip.to_string()),
            _ => None,
        });
        let name = instance.strip_suffix(&format!(".{service}")).unwrap_or(instance).replace("\\032", " ");
        let uri = format!("{scheme}://{}/", address.unwrap_or(host));
        if !servers.iter().any(|s| s.uri == uri) {
            servers.push(Server { name, uri });
        }
    }
    servers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    servers
}

// Sends the questions and collects the answers for `window`
fn collect(sock: &UdpSocket, items: &[(&str, u16)], records: &mut Vec<Record>, window: Duration) {
    let target: SocketAddr = (Ipv4Addr::new(224, 0, 0, 251), 5353).into();
    if sock.send_to(&question(items), target).is_err() {
        return;
    }
    let deadline = Instant::now() + window;
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        if let Ok((n, _)) = sock.recv_from(&mut buf) {
            records.extend(parse(&buf[..n]));
        }
    }
}

// The mDNS socket: 0.0.0.0:5353, address and port shared, member of 224.0.0.251
fn open_socket() -> Option<UdpSocket> {
    use std::os::fd::FromRawFd;
    // SAFETY: plain socket calls on a descriptor created here; it is closed on every failure path and otherwise owned by
    // the UdpSocket
    unsafe {
        let fd = libc::socket(libc::AF_INET, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0);
        if fd < 0 {
            return None;
        }
        let one: libc::c_int = 1;
        for opt in [libc::SO_REUSEADDR, libc::SO_REUSEPORT] {
            libc::setsockopt(fd, libc::SOL_SOCKET, opt, (&one as *const libc::c_int).cast(), std::mem::size_of::<libc::c_int>() as libc::socklen_t);
        }
        let addr = libc::sockaddr_in { sin_family: libc::AF_INET as libc::sa_family_t, sin_port: 5353u16.to_be(), sin_addr: libc::in_addr { s_addr: 0 }, sin_zero: [0; 8] };
        if libc::bind(fd, (&addr as *const libc::sockaddr_in).cast(), std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t) != 0 {
            libc::close(fd);
            return None;
        }
        let sock = UdpSocket::from_raw_fd(fd);
        sock.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &Ipv4Addr::UNSPECIFIED).ok()?;
        Some(sock)
    }
}

// One discovery round (about 2 to 4 seconds)
pub fn scan() -> Vec<Server> {
    let Some(sock) = open_socket().or_else(|| UdpSocket::bind("0.0.0.0:0").ok()) else { return Vec::new() };
    let _ = sock.set_read_timeout(Some(Duration::from_millis(250)));
    let _ = sock.set_multicast_ttl_v4(255);
    let mut records = Vec::new();
    let services: Vec<(&str, u16)> = SERVICES.iter().map(|(s, _)| (*s, PTR)).collect();
    collect(&sock, &services, &mut records, Duration::from_millis(1500));
    // Instances without a host, then hosts without an address: ask for them
    let instances: Vec<String> = records
        .iter()
        .filter_map(|r| match &r.data {
            Data::Ptr(i) if SERVICES.iter().any(|(s, _)| r.name == *s) => Some(i.clone()),
            _ => None,
        })
        .collect();
    let lacking: Vec<&str> = instances
        .iter()
        .filter(|i| !records.iter().any(|r| matches!(r.data, Data::Srv(_)) && r.name == i.to_lowercase()))
        .map(String::as_str)
        .collect();
    if !lacking.is_empty() {
        let items: Vec<(&str, u16)> = lacking.iter().map(|n| (*n, SRV)).collect();
        collect(&sock, &items, &mut records, Duration::from_millis(1000));
    }
    let hosts: Vec<String> = records
        .iter()
        .filter_map(|r| match &r.data {
            Data::Srv(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    let lacking: Vec<&str> = hosts.iter().filter(|h| !records.iter().any(|r| matches!(r.data, Data::A(_)) && r.name == h.to_lowercase())).map(String::as_str).collect();
    if !lacking.is_empty() {
        let items: Vec<(&str, u16)> = lacking.iter().map(|n| (*n, A)).collect();
        collect(&sock, &items, &mut records, Duration::from_millis(1000));
    }
    assemble(&records)
}

static CACHE: Mutex<Vec<Server>> = Mutex::new(Vec::new());

// The servers of the last scan
pub fn cached() -> Vec<Server> {
    CACHE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

// Scans and keeps the result; true when the list changed
pub fn refresh() -> bool {
    let found = scan();
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let changed = *cache != found;
    *cache = found;
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name_bytes(name: &str) -> Vec<u8> {
        let mut b = Vec::new();
        for l in name.split('.') {
            b.push(l.len() as u8);
            b.extend(l.as_bytes());
        }
        b.push(0);
        b
    }

    fn record(name: &str, rtype: u16, rdata: Vec<u8>) -> Vec<u8> {
        let mut b = name_bytes(name);
        b.extend(rtype.to_be_bytes());
        b.extend(1u16.to_be_bytes());
        b.extend(120u32.to_be_bytes());
        b.extend((rdata.len() as u16).to_be_bytes());
        b.extend(rdata);
        b
    }

    // A response with a PTR, its SRV and the A record of the host (as avahi and NAS boxes send them)
    fn response(instance: &str, host: &str, ip: [u8; 4]) -> Vec<u8> {
        let mut p = vec![0, 0, 0x84, 0, 0, 0, 0, 1, 0, 0, 0, 2];
        p.extend(record("_smb._tcp.local", PTR, name_bytes(instance)));
        let mut srv = vec![0, 0, 0, 0, 0x01, 0xBD];
        srv.extend(name_bytes(host));
        p.extend(record(instance, SRV, srv));
        p.extend(record(host, A, ip.to_vec()));
        p
    }

    #[test]
    fn a_full_response_gives_a_server_with_its_address() {
        let p = response("Office NAS._smb._tcp.local", "nas.local", [192, 168, 1, 20]);
        let servers = assemble(&parse(&p));
        assert_eq!(servers, vec![Server { name: "Office NAS".into(), uri: "smb://192.168.1.20/".into() }]);
    }

    #[test]
    fn without_an_address_the_host_name_is_used() {
        let mut p = vec![0, 0, 0x84, 0, 0, 0, 0, 1, 0, 0, 0, 1];
        p.extend(record("_sftp-ssh._tcp.local", PTR, name_bytes("Laptop._sftp-ssh._tcp.local")));
        let mut srv = vec![0, 0, 0, 0, 0, 22];
        srv.extend(name_bytes("laptop.local"));
        p.extend(record("Laptop._sftp-ssh._tcp.local", SRV, srv));
        assert_eq!(assemble(&parse(&p)), vec![Server { name: "Laptop".into(), uri: "sftp://laptop.local/".into() }]);
    }

    #[test]
    fn compressed_names_and_garbage_are_handled() {
        // "b.local" written as a pointer to "local" at offset 2
        let buf = [3, b'a', b'b', b'c', 0xC0, 0x00];
        assert_eq!(read_name(&buf, 0).map(|(n, _)| n), None); // runs out of bytes: no panic
        let looping = [0xC0, 0x00];
        assert_eq!(read_name(&looping, 0), None);
        assert!(parse(&[1, 2, 3]).is_empty());
        assert!(parse(&[0u8; 12]).is_empty());
    }

    #[test]
    fn questions_are_well_formed() {
        let q = question(&[("_smb._tcp.local", PTR)]);
        assert_eq!(be16(&q, 4), Some(1));
        let (name, end) = read_name(&q, 12).unwrap();
        assert_eq!(name, "_smb._tcp.local");
        assert_eq!((be16(&q, end), be16(&q, end + 2)), (Some(PTR), Some(1)));
    }

    #[test]
    fn two_services_and_duplicates() {
        let mut records = parse(&response("B Box._smb._tcp.local", "b.local", [10, 0, 0, 2]));
        records.extend(parse(&response("A Box._smb._tcp.local", "a.local", [10, 0, 0, 1])));
        records.extend(parse(&response("A Box._smb._tcp.local", "a.local", [10, 0, 0, 1])));
        let names: Vec<String> = assemble(&records).into_iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["A Box", "B Box"]);
    }
}
