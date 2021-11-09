use getopts::{Matches, Options};
use sdusrun::{read_config_from_file, select_ip, SrunClient, User};
use std::env;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} ACTION [options]\n\nActions: login", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "server", "auth server", "");
    opts.optopt("c", "config", "config file path", "");
    opts.optflag("", "continue", "[Unimplemented] continuous login");
    // login
    opts.optopt("u", "username", "username", "");
    opts.optopt("p", "password", "password", "");
    opts.optopt("i", "ip", "ip", "");
    opts.optflag("d", "detect", "detect client ip");
    opts.optflag("", "select-ip", "select client ip");
    opts.optflag("", "strict-bind", "strict bind ip");
    opts.optflag("", "test", "test network connection before login");

    opts.optopt("", "acid", "acid", "");
    opts.optopt("", "os", "os, e.g. Windows", "");
    opts.optopt("", "name", "name, e.g. Windows 98", "");
    opts.optopt("", "retry-delay", "retry delay, default 300 millis", "");
    opts.optopt("", "retry-times", "retry times, default 10 times", "");

    if args.len() < 2 {
        print_usage(&program, &opts);
        return;
    }

    let command = args[1].clone();
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => m,
        Err(e) => {
            println!("parse args error: {}", e);
            return;
        }
    };

    match command.as_str() {
        "login" => {
            if matches.opt_present("c") {
                config_login(matches);
            } else {
                single_login(matches);
            }
        }
        _ => {
            print_usage(&program, &opts);
        }
    }
}

fn config_login(matches: Matches) {
    let config_path = matches.opt_str("c").unwrap();
    match read_config_from_file(config_path) {
        Ok(config) => {
            let config_i = config.clone();
            let server = config
                .server
                .clone()
                .unwrap_or_else(|| match matches.opt_str("s") {
                    Some(u) => u,
                    None => "http://202.194.15.87".to_string(),
                });
            for user in config_i {
                println!("login user: {:#?}", user);
                let mut client = SrunClient::new_from_user(&server, user)
                    .set_strict_bind(config.strict_bind)
                    .set_double_stack(config.double_stack);
                if let Some(acid) = config.acid {
                    client.set_acid(acid);
                }
                if let Some(ref os) = config.os {
                    client.set_os(os);
                }
                if let Some(ref name) = config.name {
                    client.set_name(name);
                }
                if let Some(retry_delay) = config.retry_delay {
                    client.set_retry_delay(retry_delay);
                }
                if let Some(retry_times) = config.retry_times {
                    client.set_retry_times(retry_times);
                }

                if let Err(e) = client.login() {
                    println!("login error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("read config file error: {}", e);
        }
    }
}

fn single_login(matches: Matches) {
    let auth_server = match matches.opt_str("s") {
        Some(u) => u,
        None => "http://202.194.15.87".to_string(),
    };
    let username = match matches.opt_str("u") {
        Some(u) => u,
        None => {
            println!("need username");
            return;
        }
    };
    let password = match matches.opt_str("p") {
        Some(u) => u,
        None => {
            println!("need password");
            return;
        }
    };
    let detect_ip = matches.opt_present("d");
    let ip = match matches.opt_str("i") {
        Some(u) => u,
        None => {
            if matches.opt_present("select-ip") {
                select_ip().unwrap_or_default()
            } else if detect_ip {
                String::new()
            } else {
                println!("need ip");
                println!("  1. use '-i IP' to specify ip");
                println!("  2. use '-d' to auto detect ip");
                println!("  3. use '--select-ip' to select ip");
                return;
            }
        }
    };
    let test = matches.opt_present("test");
    let strict_bind = matches.opt_present("strict-bind");

    let user = User {
        username,
        password,
        ip: Some(ip),
        if_name: None,
    };
    println!("login user: {:#?}", user);
    let mut client = SrunClient::new_from_user(&auth_server, user)
        .set_detect_ip(detect_ip)
        .set_test_before_login(test)
        .set_strict_bind(strict_bind);

    if let Some(acid) = matches.opt_str("acid") {
        client.set_acid(acid.parse().unwrap());
    }

    if let Some(ref os) = matches.opt_str("os") {
        client.set_os(os);
    }

    if let Some(ref name) = matches.opt_str("name") {
        client.set_name(name);
    }

    if let Some(retry_delay) = matches.opt_str("retry-delay") {
        client.set_retry_delay(retry_delay.parse().unwrap_or(300));
    }

    if let Some(retry_times) = matches.opt_str("retry-times") {
        client.set_retry_times(retry_times.parse().unwrap_or(10));
    }

    if let Err(e) = client.login() {
        println!("login error: {}", e);
    }
}
