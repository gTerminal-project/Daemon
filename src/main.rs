extern crate r_i18n;

mod servers;

use r_i18n::{I18nConfig, I18n};
use current_locale;
use servers::HTTPServer;
use std::env;
use rand::Rng;
use base32;
use keyring::Entry;
use dyn_fmt::AsStrFormatExt;

#[tokio::main]
async fn main() {
    // Initialize keyring
    let entry = Entry::new("gTerminal-Daemon", "TOTP-Secret").unwrap();

    // Get cli arguments
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        if args[1] == "pair" {
            // Generate TOTP secret & save it to keyring
            let totp_secret = generate_totp_secret();
            entry.set_password(&totp_secret).expect(&get_string("couldNotSaveTOTPSecret"));

            println!("{}", get_string("pairInstructions").format(&[totp_secret]));
        } else if args[1] == "listen" {
            // Init & start HTTP server
            let http_server = HTTPServer::new(entry);
            http_server.serve();
        } else {
            // Show help
            help();
        }
    } else {
        // Show help
        help();
    }
}

fn generate_totp_secret() -> String {
    let mut rng = rand::thread_rng();
    let data_byte: [u8; 21] = rng.gen();
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, &data_byte)
}

fn get_string(key: &str) -> String {
    let config =  I18nConfig{locales: &["en", "de"], directory: "translations"};
    let mut r_i18n = I18n::configure(&config);

    let lang = current_locale::current_locale().unwrap().split_once("-").unwrap().0.to_string();
    if config.locales.contains(&lang.as_str()) {
        r_i18n.set_current_lang(&lang);
    }
    r_i18n.t(key).to_string()
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