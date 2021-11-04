use std::{io, net::IpAddr};

fn get_ips() -> Vec<IpAddr> {
    pnet_datalink::interfaces()
        .iter()
        .filter(|i| i.is_up() && i.is_multicast())
        .map(|i| -> Vec<IpAddr> {
            i.ips
                .iter()
                .filter(|ip| ip.is_ipv4())
                .map(|ip| ip.ip())
                .collect()
        })
        .flatten()
        .collect()
}

pub fn select_ip() -> Option<String> {
    let ips = get_ips();
    if ips.is_empty() {
        return None;
    }
    if ips.len() == 1 {
        return Some(ips[0].to_string());
    }

    println!("Please select your IP:");
    for (n, ip) in ips.iter().enumerate() {
        println!("    {}. {}", n + 1, ip);
    }

    for _ in 0..10 {
        let mut input_text = String::new();
        io::stdin()
            .read_line(&mut input_text)
            .expect("failed to read from stdin");
        let trimmed = input_text.trim();
        if let Ok(i) = trimmed.parse::<usize>() {
            if i > 0 && i <= ips.len() {
                let ip = ips[i - 1].to_string();
                println!("you choose {}", ip);
                return Some(ip);
            }
        }
        println!("not a valid index number");
    }
    println!("invalid input for 10 times");
    None
}

#[test]
fn test_get_ips() {
    select_ip();
}
