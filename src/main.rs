use std::env;
use rand::Rng;
use base32;
use std::time::SystemTime;
use totp_rs::{Algorithm, TOTP, Secret};
use keyring::Entry;
use rouille::Response;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    command: String,
    totp: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Result {
    stdout: String,
    stderr: String,
    success: bool
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let entry = Entry::new("gTerminal-Daemon", "TOTP-Secret").unwrap();

    if args.len() == 2 {
        if args[1] == "pair" {
            let totp_secret = generate_totp_secret();
            entry.set_password(&totp_secret).expect("Could not save TOTP-Secret");
            println!("Go to gTerminal and run \"pair {}\"", totp_secret);
        } else if args[1] == "listen" {
            rouille::start_server("127.0.0.1:10384", move | request| {
                if request.method() == "POST" {
                    let data = request.data().unwrap();

                    let data: Data = serde_json::from_reader(data).unwrap();

                    if check_totp_code(&entry.get_password().unwrap(), &data.totp) {
                        let command: Vec<&str> = data.command.split_whitespace().collect();
                        let output = Command::new(&command[0])
                            .args(&command[1..])
                            .output()
                            .expect("Error");

                        let result: Result = Result { stdout: String::from_utf8_lossy(&output.stdout).to_string(), stderr: String::from_utf8_lossy(&output.stderr).to_string(), success: true };
                        Response::text(serde_json::to_string(&result).unwrap())
                    } else {
                        Response::text("{\"content\": \"Invalid TOTP Code\", \"success\": false}")
                    }
                } else {
                    Response::text("{\"content\": \"Method not allowed\", \"success\": false}")
                }
            });
        } else {
            help();
        }
    } else {
        help()
    }
}

fn check_totp_code(secret: &str, token: &str) -> bool {
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, 
        Secret::Encoded(secret.to_string())
        .to_bytes().unwrap())
        .unwrap();
    (totp.generate_current().unwrap() == token) | (totp.generate(get_unix_time() - totp.step) == token)
}

fn generate_totp_secret() -> String {
    let mut rng = rand::thread_rng();
    let data_byte: [u8; 21] = rng.gen();
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, &data_byte)
}

fn get_unix_time() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

fn help() {
    let args: Vec<String> = env::args().collect();

    println!("{} {}", args[0], env!("CARGO_PKG_VERSION"));
    println!("{}", env!("CARGO_PKG_AUTHORS"));
    println!("Usage: {} <SUBCOMMAND>", args[0]);
    println!("");
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    println!("");
    println!("SUBCOMMANDS:");
	println!("    pair      Pair with gTerminal");
	println!("    listen    Start the daemon");
}