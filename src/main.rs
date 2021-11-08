use getopts::{Matches, Options};
use sdusrun::{mng::LoginMng, read_config_from_file, select_ip, User};
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
    opts.optflag("", "continue", "continuous login");
    // login
    opts.optopt("u", "username", "username", "");
    opts.optopt("p", "password", "password", "");
    opts.optopt("i", "ip", "ip", "");
    opts.optflag("d", "detect", "detect client ip");
    opts.optflag("", "select-ip", "select client ip");
    opts.optflag("", "strict-bind", "strict bind ip");
    opts.optflag(
        "",
        "test",
        "test before login, only avaiable for single user",
    );

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
            let strict_bind = config.strict_bind;
            let server = config
                .server
                .clone()
                .unwrap_or_else(|| match matches.opt_str("s") {
                    Some(u) => u,
                    None => "http://202.194.15.87".to_string(),
                });
            for user in config {
                login(&server, user, false, false, strict_bind)
            }
        }
        Err(e) => {
            println!("read config file error: {}", e);
        }
    }
}

fn single_login(matches: Matches) {
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
    login(
        &server,
        User::new(username, password, ip),
        detect_ip,
        test,
        strict_bind,
    );
}

fn login(auth_server: &str, user: User, detect_ip: bool, test: bool, strict_bind: bool) {
    println!("login user: {:#?}", user);
    let mut mng = LoginMng::new(auth_server.to_owned(), user)
        .set_detect_ip(detect_ip)
        .set_test_before_login(test)
        .set_strict_bind(strict_bind);
    if let Err(e) = mng.login_once() {
        println!("login error: {}", e);
    }
}
