use crate::{utils::get_ip_by_if_name, User};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
pub struct LoginMng {
    auth_server: String,
    test: bool,

    username: String,
    password: String,
    ip: String,
    detect_ip: bool,
}

impl LoginMng {
    pub fn new(auth_server: String, user: User) -> Self {
        let ip = user
            .ip
            .unwrap_or_else(|| get_ip_by_if_name(&user.if_name.unwrap()).unwrap_or_default());
        Self {
            auth_server,
            test: true,

            username: user.username,
            password: user.password,
            ip,
            detect_ip: false,
        }
    }

    pub fn set_detect_ip(mut self, detect_ip: bool) -> Self {
        self.detect_ip = detect_ip;
        self
    }

    pub fn set_test_before_login(mut self, test: bool) -> Self {
        self.test = test;
        self
    }

    pub fn login_once(&mut self) -> Result<()> {
        let mut client =
            crate::SrunClient::new(&self.auth_server, &self.username, &self.password, &self.ip)
                .set_detect_ip(self.detect_ip);
        if let Err(e) = client.login() {
            println!("login error: {}", e);
        }
        Ok(())
    }
}
