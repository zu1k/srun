use std::env;

use getopts::{Matches, Options};

use srun::{get_ip_by_if_name, read_config_from_file, select_ip, SrunClient, User};

fn print_usage(opts: Option<&Options>) {
    let brief = "Usage: srun ACTION [options]\n\nActions: login | logout".to_string();
    if let Some(opts) = opts {
        print!("{}", opts.usage(&brief));
    } else {
        println!("{}", brief);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(None);
        return;
    }

    match args[1].as_str() {
        "login" => login_match(&args),
        "logout" => logout_match(&args),
        _ => {
            print_usage(None);
        }
    }
}

fn login_match(args: &[String]) {
    let options = {
        let mut opts = Options::new();
        opts.optflag("h", "help", "print help message");
        opts.optopt("s", "server", "auth server", "");
        opts.optopt("c", "config", "config file path", "");
        opts.optflag("", "continue", "[Unimplemented] continuous login");
        opts.optopt("u", "username", "username", "");
        opts.optopt("p", "password", "password", "");
        opts.optopt("i", "ip", "ip", "");
        opts.optflag("d", "detect", "detect client ip");
        opts.optflag("", "select-ip", "select client ip");
        opts.optflag("", "strict-bind", "strict bind ip");
        opts.optflag("", "test", "test network connection before login");
        opts.optflag("", "double-stack", "enable double stack");
        opts.optopt("n", "param-n", "n", "");
        opts.optopt("", "type", "type", "");
        opts.optopt("", "acid", "acid", "");
        opts.optopt("", "os", "os, e.g. Windows", "");
        opts.optopt("", "name", "name, e.g. Windows 98", "");
        opts.optopt("", "retry-delay", "retry delay, default 300 millis", "");
        opts.optopt("", "retry-times", "retry times, default 10 times", "");
        opts
    };

    let matches = match options.parse(args) {
        Ok(m) => m,
        Err(e) => {
            println!("parse args error: {}", e);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(Some(&options));
    } else if matches.opt_present("c") {
        config_login(matches);
    } else {
        single_login(matches);
    }
}

fn logout_match(args: &[String]) {
    let options = {
        let mut opts = Options::new();
        opts.optflag("h", "help", "print help message");
        opts.optopt("s", "server", "auth server", "");
        opts.optopt("u", "username", "username", "");
        opts.optopt("i", "ip", "ip", "");
        opts.optflag("d", "detect", "detect client ip");
        opts.optopt("c", "config", "logout by config file", "");
        opts.optflag("", "select-ip", "select client ip");
        opts.optflag("", "strict-bind", "strict bind ip");
        opts.optopt("", "acid", "acid", "");
        opts
    };

    let matches = match options.parse(args) {
        Ok(m) => m,
        Err(e) => {
            println!("parse args error: {}", e);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(Some(&options));
    } else if matches.opt_present("c") {
        config_logout(matches);
    } else {
        logout(matches)
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
                    None => format!("http://{}", env!("AUTH_SERVER_IP")),
                });
            for user in config_i {
                println!("login user: {:#?}", user);
                let mut client = SrunClient::new_from_user(&server, user)
                    .set_detect_ip(config.detect_ip)
                    .set_strict_bind(config.strict_bind)
                    .set_double_stack(config.double_stack);
                if let Some(n) = config.n {
                    client.set_n(n);
                }
                if let Some(utype) = config.utype {
                    client.set_type(utype);
                }
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
        None => format!("http://{}", env!("AUTH_SERVER_IP")),
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
        .set_strict_bind(strict_bind)
        .set_double_stack(matches.opt_present("double-stack"));

    if let Some(n) = matches.opt_str("n") {
        client.set_n(n.parse().unwrap());
    }

    if let Some(utype) = matches.opt_str("type") {
        client.set_type(utype.parse().unwrap());
    }

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
        client.set_retry_delay(retry_delay.parse().unwrap_or(1000));
    }

    if let Some(retry_times) = matches.opt_str("retry-times") {
        client.set_retry_times(retry_times.parse().unwrap_or(3));
    }

    if let Err(e) = client.login() {
        println!("login error: {}", e);
    }
}

fn config_logout(matches: Matches) {
    let config_path = matches.opt_str("c").unwrap();
    match read_config_from_file(config_path) {
        Ok(config) => {
            let config_i = config.clone();
            let auth_server = config
                .server
                .clone()
                .unwrap_or_else(|| match matches.opt_str("s") {
                    Some(u) => u,
                    None => format!("http://{}", env!("AUTH_SERVER_IP")),
                });
            for user in config_i {
                println!("logout user: {:#?}", user);
                let ip = user.ip.unwrap_or_else(|| {
                    get_ip_by_if_name(&user.if_name.unwrap_or_default()).unwrap_or_default()
                });
                let mut client = SrunClient::new_for_logout(&auth_server, &user.username, &ip)
                    .set_detect_ip(config.detect_ip)
                    .set_strict_bind(config.strict_bind);

                if let Some(acid) = config.acid {
                    client.set_acid(acid);
                }

                if let Err(e) = client.logout() {
                    println!("logout error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("read config file error: {}", e);
        }
    }
}

fn logout(matches: Matches) {
    let auth_server = match matches.opt_str("s") {
        Some(u) => u,
        None => format!("http://{}", env!("AUTH_SERVER_IP")),
    };
    let username = match matches.opt_str("u") {
        Some(u) => u,
        None => {
            println!("need username");
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
    let strict_bind = matches.opt_present("strict-bind");
    let mut client = SrunClient::new_for_logout(&auth_server, &username, &ip)
        .set_detect_ip(detect_ip)
        .set_strict_bind(strict_bind);

    if let Some(acid) = matches.opt_str("acid") {
        client.set_acid(acid.parse().unwrap());
    }

    if let Err(e) = client.logout() {
        println!("logout error: {}", e);
    }
}
