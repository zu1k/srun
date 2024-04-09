use std::{env, net::Ipv4Addr};

fn main() {
    let auth_server_ip =
        env!(
        "AUTH_SERVER_IP",
        "Expect env AUTH_SERVER_IP, export AUTH_SERVER_IP=10.0.0.1"
    );
    println!("ENV AUTH_SERVER_IP = {auth_server_ip}");
    auth_server_ip
        .parse::<Ipv4Addr>()
        .expect("AUTH_SERVER_IP invalid");
}
