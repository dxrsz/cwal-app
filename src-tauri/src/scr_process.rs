use netstat::TcpState;
use sysinfo::{Pid, System};

fn tcp_state_eq(state: &TcpState, other: &TcpState) -> bool {
    state.to_string() == other.to_string()
}

pub fn find_starcraft_process(system: &mut System) -> Option<Pid> {
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut starcraft_pid = system.processes_by_exact_name("StarCraft.exe".as_ref());

    starcraft_pid.next().map(|process| process.pid())
}

pub fn find_starcraft_api_port(pid: &Pid) -> Option<u16> {
    let af_flags = netstat::AddressFamilyFlags::IPV4 | netstat::AddressFamilyFlags::IPV6;
    let proto_flags = netstat::ProtocolFlags::TCP | netstat::ProtocolFlags::UDP;
    let sockets_info = netstat::get_sockets_info(af_flags, proto_flags);

    sockets_info
        .ok()?
        .iter()
        .find_map(|si| match &si.protocol_socket_info {
            netstat::ProtocolSocketInfo::Tcp(tcp_si) => {
                if si.associated_pids.contains(&pid.as_u32())
                    && tcp_state_eq(&tcp_si.state, &TcpState::Listen)
                {
                    Some(tcp_si.local_port)
                } else {
                    None
                }
            }
            _ => None,
        })
}
