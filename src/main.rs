use getopts::Options;
use sdusrun::{read_config_from_file, User};
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
    // login
    opts.optopt("u", "username", "username", "");
    opts.optopt("p", "password", "password", "");
    opts.optopt("i", "ip", "ip", "");
    opts.optflag("d", "detect", "detect client ip");

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
        "login" => match matches.opt_str("c") {
            Some(config_path) => match read_config_from_file(config_path) {
                Ok(config) => {
                    let server =
                        config
                            .server
                            .clone()
                            .unwrap_or_else(|| match matches.opt_str("s") {
                                Some(u) => u,
                                None => "http://202.194.15.87".to_string(),
                            });
                    for user in config {
                        login(&server, user, false)
                    }
                }
                Err(e) => {
                    println!("read config file error: {}", e);
                }
            },
            None => {
                let server = match matches.opt_str("s") {
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
                let ip = match matches.opt_str("i") {
                    Some(u) => u,
                    None => String::new(),
                };
                let use_auth_provided_ip = matches.opt_present("d");
                login(
                    &server,
                    User::new(username, password, ip),
                    use_auth_provided_ip,
                );
            }
        },
        _ => {
            print_usage(&program, &opts);
        }
    }
}

fn login(server: &str, user: User, use_auth_provided_ip: bool) {
    println!("login user: {:#?}", user);
    let mut client =
        sdusrun::SrunClient::new_from_info(server, user).set_auto_detect_ip(use_auth_provided_ip);
    if let Err(e) = client.login() {
        println!("login error: {}", e);
    }
}
