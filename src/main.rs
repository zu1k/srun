fn main() {
    let mut client =
        sdusrun::SrunClient::new("http://auth_gateway", "username", "password", "your ip");
    client.login().expect("login err");
}
