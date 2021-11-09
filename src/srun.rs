use crate::{param_i, utils::get_ip_by_if_name, Result, User};
use hmac::{Hmac, Mac, NewMac};
use md5::Md5;
use reqwest::blocking::{Client, ClientBuilder};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::{
    net::IpAddr,
    str::FromStr,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const PATH_GET_CHALLENGE: &str = "/cgi-bin/get_challenge";
const PATH_LOGIN: &str = "/cgi-bin/srun_portal";

#[derive(Default, Debug)]
pub struct SrunClient {
    auth_server: String,

    username: String,
    password: String,
    ip: String,
    detect_ip: bool,
    strict_bind: bool,

    retry_delay: u32, // millis
    retry_times: u32,
    test_before_login: bool,

    acid: i32,
    double_stack: i32,
    os: String,
    name: String,

    token: String,
    n: i32,
    stype: i32,
    time: u64,
}

quick_error! {
    #[derive(Debug)]
    pub enum SrunError {
        GetChallengeFailed
        IpUndefinedError
    }
}

impl SrunClient {
    pub fn new_from_user(auth_server: &str, user: User) -> Self {
        let ip = user
            .ip
            .unwrap_or_else(|| get_ip_by_if_name(&user.if_name.unwrap()).unwrap_or_default());
        Self {
            auth_server: auth_server.to_owned(),
            username: user.username,
            password: user.password,
            ip,
            acid: 12,
            n: 200,
            stype: 1,
            os: "Windows 10".to_string(),
            name: "Windows".to_string(),
            retry_delay: 300,
            retry_times: 10,
            ..Default::default()
        }
    }

    pub fn set_detect_ip(mut self, b: bool) -> Self {
        self.detect_ip = b;
        self
    }

    pub fn set_strict_bind(mut self, b: bool) -> Self {
        self.strict_bind = b;
        self
    }

    pub fn set_double_stack(mut self, b: bool) -> Self {
        self.double_stack = b as i32;
        self
    }

    pub fn set_acid(&mut self, acid: i32) {
        self.acid = acid;
    }

    pub fn set_os(&mut self, os: &str) {
        self.os = os.to_owned();
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn set_retry_delay(&mut self, d: u32) {
        self.retry_delay = d;
    }

    pub fn set_retry_times(&mut self, t: u32) {
        self.retry_times = t;
    }

    pub fn set_test_before_login(mut self, b: bool) -> Self {
        self.test_before_login = b;
        self
    }

    pub fn get_http_client(&self) -> Result<Client> {
        Ok(if self.strict_bind && !self.ip.is_empty() {
            let local_addr = IpAddr::from_str(&self.ip)?;
            ClientBuilder::default()
                .local_address(local_addr)
                .connect_timeout(Duration::from_secs(3))
                .build()?
        } else {
            Client::default()
        })
    }

    fn get_token(&mut self) -> Result<String> {
        if !self.detect_ip && self.ip.is_empty() {
            println!("need ip");
            return Err(Box::new(SrunError::IpUndefinedError));
        }

        self.time = unix_second() - 2;
        let resp = self
            .get_http_client()?
            .get(format!("{}{}", self.auth_server, PATH_GET_CHALLENGE).as_str())
            .query(&vec![
                ("callback", "sdu"),
                ("username", &self.username),
                ("ip", &self.ip),
                ("_", &self.time.to_string()),
            ])
            .send()?
            .bytes()?;

        let challenge: ChallengeResponse = serde_json::from_slice(&resp[4..resp.len() - 1])?;
        println!("{:#?}", challenge);
        match challenge.challenge.clone() {
            Some(token) => {
                self.token = token;
                if self.detect_ip && !challenge.client_ip.is_empty() {
                    self.ip = challenge.client_ip;
                }
            }
            None => {
                return Err(Box::new(SrunError::GetChallengeFailed));
            }
        };
        Ok(self.token.clone())
    }

    pub fn login(&mut self) -> Result<()> {
        self.get_token()?;

        if self.ip.is_empty() {
            return Err(Box::new(SrunError::IpUndefinedError));
        }

        let hmd5 = {
            let mut mac = Hmac::<Md5>::new_from_slice(self.token.as_bytes()).expect("aa");
            mac.update(self.password.as_bytes());
            let result = mac.finalize();
            format!("{:x}", result.into_bytes())
        };

        let param_i = param_i(
            &self.username,
            &self.password,
            &self.ip,
            self.acid,
            &self.token,
        );

        let check_sum = {
            let check_sum = vec![
                "",
                &self.username,
                &hmd5,
                &self.acid.to_string(),
                &self.ip,
                &self.n.to_string(),
                &self.stype.to_string(),
                &param_i,
            ]
            .join(&self.token);
            let mut sha1_hasher = Sha1::new();
            sha1_hasher.update(check_sum);
            format!("{:x}", sha1_hasher.finalize())
        };

        println!("will try at most {} times...", self.retry_times);
        let mut result = LoginResponse::default();
        for ti in 1..=self.retry_times {
            let resp = self
                .get_http_client()?
                .get(format!("{}{}", self.auth_server, PATH_LOGIN).as_str())
                .query(&vec![
                    ("callback", "sdu"),
                    ("action", "login"),
                    ("username", &self.username),
                    ("password", &format!("{{MD5}}{}", hmd5)),
                    ("ip", &self.ip),
                    ("ac_id", self.acid.to_string().as_str()),
                    ("n", self.n.to_string().as_str()),
                    ("type", self.stype.to_string().as_str()),
                    ("os", &self.os),
                    ("name", &self.name),
                    ("double_stack", &self.double_stack.to_string()),
                    ("info", &param_i),
                    ("chksum", &check_sum),
                    ("_", &self.time.to_string()),
                ])
                .send()?
                .bytes()?;
            result = serde_json::from_slice(&resp[4..resp.len() - 1])?;
            if !result.access_token.is_empty() {
                println!("try {}/{}: success\n{:#?}", ti, self.retry_times, result);
                return Ok(());
            }
            println!("try {}/{}: failed", ti, self.retry_times);
            thread::sleep(Duration::from_millis(self.retry_delay as u64));
        }
        println!("{:#?}", result);
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
struct ChallengeResponse {
    challenge: Option<String>,
    client_ip: String,
    ecode: ECode,
    error_msg: String,
    expire: Option<String>,
    online_ip: String,
    res: String,
    srun_ver: String,
    st: u64,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LoginResponse {
    #[serde(rename(deserialize = "ServerFlag"))]
    server_flag: i32,
    #[serde(rename(deserialize = "ServicesIntfServerIP"))]
    services_intf_server_ip: String,
    #[serde(rename(deserialize = "ServicesIntfServerPort"))]
    services_intf_server_port: String,
    access_token: String,
    checkout_date: u64,
    ecode: ECode,
    error: String,
    error_msg: String,
    client_ip: String,
    online_ip: String,
    real_name: String,
    remain_flux: i32,
    remain_times: i32,
    res: String,
    srun_ver: String,
    suc_msg: String,
    sysver: String,
    username: String,
    wallet_balance: i32,
    st: u64,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ECode {
    I(i32),
    S(String),
}

impl Default for ECode {
    fn default() -> Self {
        Self::I(0)
    }
}

fn unix_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}
