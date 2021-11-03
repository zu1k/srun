use getopts::Options;
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
    // login
    opts.optopt("u", "username", "username", "");
    opts.optopt("p", "password", "password", "");
    opts.optopt("i", "ip", "ip", "");

    if args.len() < 2 {
        print_usage(&program, &opts);
        return;
    }

    let command = args[1].clone();
    match command.as_str() {
        "login" => {
            println!("do login");
            let matches = match opts.parse(&args[2..]) {
                Ok(m) => m,
                Err(e) => {
                    println!("parse args error: {}", e);
                    return;
                }
            };
            login(
                matches.opt_str("s"),
                matches.opt_str("u"),
                matches.opt_str("p"),
                matches.opt_str("i"),
            );
        }
        _ => {
            print_usage(&program, &opts);
        }
    }
}

fn login(
    server: Option<String>,
    username: Option<String>,
    password: Option<String>,
    ip: Option<String>,
) {
    let server = match server {
        Some(u) => u,
        None => "202.194.15.87".to_string(),
    };
    let username = match username {
        Some(u) => u,
        None => {
            println!("need username");
            return;
        }
    };
    let password = match password {
        Some(u) => u,
        None => {
            println!("need password");
            return;
        }
    };
    let ip = match ip {
        Some(u) => u,
        None => {
            println!("need ip");
            return;
        }
    };
    let mut client = sdusrun::SrunClient::new(&server, &username, &password, &ip);
    client.login().expect("login err");
}
