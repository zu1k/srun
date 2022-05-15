use crate::Result;
use std::{
    io,
    net::{IpAddr, TcpStream, ToSocketAddrs},
    process::exit,
    time::{Duration, SystemTime},
};

quick_error! {
    #[derive(Debug)]
    pub enum UtilError {
        AddrResolveError
    }
}

pub fn tcp_ping(addr: &str) -> Result<u16> {
    let addr = addr.to_socket_addrs()?.next();
    if addr.is_none() {
        return Err(Box::new(UtilError::AddrResolveError));
    }
    let timeout = Duration::from_secs(3);
    let start_time = SystemTime::now();
    let stream = TcpStream::connect_timeout(&addr.unwrap(), timeout)?;
    stream.peer_addr()?;
    let d = SystemTime::now().duration_since(start_time)?;
    Ok(d.as_millis() as u16)
}

fn get_ifs() -> Vec<(String, IpAddr)> {
    let ifs = match if_addrs::get_if_addrs() {
        Ok(ifs) => ifs,
        Err(err) => {
            println!("Get Net Intercafes failed: {err}");
            exit(500)
        }
    };

    let mut ips = Vec::with_capacity(ifs.len());
    for i in ifs {
        if !i.is_loopback() {
            ips.push((i.name.clone(), i.ip()))
        }
    }
    ips
}

pub fn get_ip_by_if_name(if_name: &str) -> Option<String> {
    let ifs = get_ifs();
    for i in ifs {
        if i.0.contains(if_name) {
            return Some(i.1.to_string());
        }
    }
    None
}

pub fn select_ip() -> Option<String> {
    let ips = get_ifs();
    if ips.is_empty() {
        return None;
    }
    if ips.len() == 1 {
        return Some(ips[0].1.to_string());
    }

    println!("Please select your IP:");
    for (n, ip) in ips.iter().enumerate() {
        println!("    {}. {}", n + 1, ip.1);
    }

    for t in 1..=3 {
        let mut input_text = String::new();
        io::stdin()
            .read_line(&mut input_text)
            .expect("failed to read from stdin");
        let trimmed = input_text.trim();
        if let Ok(i) = trimmed.parse::<usize>() {
            if i > 0 && i <= ips.len() {
                let ip = ips[i - 1].1.to_string();
                println!("you choose {}", ip);
                return Some(ip);
            }
        }
        println!("not a valid index number, {}/3", t);
    }
    println!("invalid input for 3 times");
    None
}

#[test]
fn test_get_ips() {
    select_ip();
}

#[test]
fn test_get_ip_by_name() {
    println!("{:?}", get_ip_by_if_name("wlp3s0"));
}

#[test]
fn test_tcp_ping() {
    let p = tcp_ping("baidu.com:80");
    println!("{:?}", p);
}
