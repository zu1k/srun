use crate::param_i;
use hmac::{Hmac, Mac, NewMac};
use md5::Md5;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const PATH_GET_CHALLENGE: &str = "/cgi-bin/get_challenge";
const PATH_LOGIN: &str = "/cgi-bin/srun_portal";

#[derive(Default, Debug)]
pub struct SrunClient {
    server: String,
    use_auth_provided_ip: bool,

    username: String,
    password: String,
    ip: String,
    acid: i32,
    token: String,
    n: i32,
    stype: i32,
    double_stack: i32,
    os: String,
    name: String,
    time: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
    pub ip: String,
}

impl User {
    pub fn new(username: String, password: String, ip: String) -> Self {
        Self {
            username,
            password,
            ip,
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum SrunError {
        GetChallengeFailed
        IpUndefinedError
    }
}

impl SrunClient {
    pub fn new_from_info(host: &str, info: User) -> Self {
        Self {
            username: info.username,
            password: info.password,
            ip: info.ip,
            server: host.to_string(),
            acid: 12,
            n: 200,
            stype: 1,
            double_stack: 0,
            os: "Windows 10".to_string(),
            name: "Windows".to_string(),
            ..Default::default()
        }
    }

    pub fn set_auto_detect_ip(mut self, detect: bool) -> Self {
        self.use_auth_provided_ip = detect;
        self
    }

    fn get_token(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.use_auth_provided_ip && self.ip.is_empty() {
            self.ip = crate::select_ip().unwrap_or_default();
            if self.ip.is_empty() {
                println!("need ip");
                return Err(Box::new(SrunError::IpUndefinedError));
            }
        }

        self.time = unix_second() - 2;
        let resp = ureq::get(format!("{}{}", self.server, PATH_GET_CHALLENGE).as_str())
            .query("callback", "sdu")
            .query("username", &self.username)
            .query("ip", &self.ip)
            .query("_", &self.time.to_string())
            .call()?
            .into_string()?;
        let resp = resp.as_bytes();

        let challenge: ChallengeResponse = serde_json::from_slice(&resp[4..resp.len() - 1])?;
        println!("{:#?}", challenge);
        match challenge.challenge.clone() {
            Some(token) => {
                self.token = token;
                if self.use_auth_provided_ip && !challenge.client_ip.is_empty() {
                    self.ip = challenge.client_ip;
                }
            }
            None => {
                return Err(Box::new(SrunError::GetChallengeFailed));
            }
        };
        Ok(self.token.clone())
    }

    pub fn login(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        println!("will try at most 10 times...");
        let mut result = LoginResponse::default();
        for ti in 1..=10 {
            let resp = ureq::get(format!("{}{}", self.server, PATH_LOGIN).as_str())
                .query("callback", "sdu")
                .query("action", "login")
                .query("username", &self.username)
                .query("password", format!("{{MD5}}{}", hmd5).as_str())
                .query("ip", &self.ip)
                .query("ac_id", self.acid.to_string().as_str())
                .query("n", self.n.to_string().as_str())
                .query("type", self.stype.to_string().as_str())
                .query("os", &self.os)
                .query("name", &self.name)
                .query("double_stack", self.double_stack.to_string().as_str())
                .query("info", &param_i)
                .query("chksum", &check_sum)
                .query("_", &self.time.to_string())
                .call()?
                .into_string()?;

            let resp = resp.as_bytes();
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
