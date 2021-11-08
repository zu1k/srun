use crate::{param_i, Result};
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
pub struct SrunClient<'s> {
    auth_server: &'s str,

    username: &'s str,
    password: &'s str,
    ip: String,
    detect_ip: bool,
    strict_bind: bool,

    acid: i32,
    token: String,
    n: i32,
    stype: i32,
    double_stack: i32,
    os: String,
    name: String,
    time: u64,
}

quick_error! {
    #[derive(Debug)]
    pub enum SrunError {
        GetChallengeFailed
        IpUndefinedError
    }
}

impl<'s> SrunClient<'s> {
    pub fn new(auth_server: &'s str, username: &'s str, password: &'s str, ip: &'s str) -> Self {
        Self {
            username,
            password,
            ip: ip.to_owned(),
            auth_server,
            acid: 12,
            n: 200,
            stype: 1,
            double_stack: 0,
            os: "Windows 10".to_string(),
            name: "Windows".to_string(),
            ..Default::default()
        }
    }

    pub fn set_detect_ip(mut self, detect: bool) -> Self {
        self.detect_ip = detect;
        self
    }

    pub fn set_strict_bind(mut self, strict_bind: bool) -> Self {
        self.strict_bind = strict_bind;
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
                ("username", self.username),
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
            self.username,
            self.password,
            &self.ip,
            self.acid,
            &self.token,
        );

        let check_sum = {
            let check_sum = vec![
                "",
                self.username,
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

        println!("will try at most 10 times...");
        let mut result = LoginResponse::default();
        for ti in 1..=10 {
            let resp = self
                .get_http_client()?
                .get(format!("{}{}", self.auth_server, PATH_LOGIN).as_str())
                .query(&vec![
                    ("callback", "sdu"),
                    ("action", "login"),
                    ("username", self.username),
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
                println!("try {}: success\n{:#?}", ti, result);
                return Ok(());
            }
            println!("try {}: failed", ti);
            thread::sleep(Duration::from_millis(300));
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
