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
    host: String,
    challenge: ChallengeResponse,

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

impl SrunClient {
    pub fn new(host: &str, username: &str, password: &str, ip: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            ip: ip.to_string(),
            host: host.to_string(),
            acid: 12,
            n: 200,
            stype: 1,
            double_stack: 0,
            os: "Windows 10".to_string(),
            name: "Windows".to_string(),
            ..Default::default()
        }
    }

    fn get_token(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        self.time = unix_second() - 2;
        let resp = ureq::get(format!("http://{}{}", self.host, PATH_GET_CHALLENGE).as_str())
            .query("callback", "sdu")
            .query("username", &self.username)
            .query("ip", &self.ip)
            .query("_", &self.time.to_string())
            .call()?
            .into_string()?;
        let resp = resp.as_bytes();

        self.challenge = serde_json::from_slice(&resp[4..resp.len() - 1])?;
        println!("{:#?}", &self.challenge);
        self.token = self.challenge.challenge.clone();
        Ok(self.token.clone())
    }

    pub fn login(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.get_token()?;

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
            let resp = ureq::get(format!("http://{}{}", self.host, PATH_LOGIN).as_str())
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
        println!("{:?}", result);
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
struct ChallengeResponse {
    challenge: String,
    client_ip: String,
    ecode: i32,
    error_msg: String,
    expire: String,
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
    ecode: i32,
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

fn unix_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}
