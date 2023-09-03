use rouille::Response;
use std::process::Command;
use totp_rs::{Algorithm, TOTP, Secret};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::env;
use keyring::Entry;
use dyn_fmt::AsStrFormatExt;

#[derive(Serialize, Deserialize)]
struct RequestData {
    command: String,
    totp: String,
}

#[derive(Serialize, Deserialize)]
struct ResultData {
    stdout: String,
    stderr: String,
    command_success: bool,
    success: bool
}

pub struct HTTPServer {
    entry: Entry,
 }

impl HTTPServer {
    pub fn new(entry: Entry) -> HTTPServer {
        HTTPServer { entry: entry }
    }

    pub fn serve(self) {
        rouille::start_server("127.0.0.1:10384", move | request| {
            if request.method() == "POST" {
                // Get request’s JSON data
                let data: RequestData = serde_json::from_reader(request.data().unwrap()).expect(&crate::get_string("couldNotParseReqData"));

                // Get binary name
                let binary_name: String = env::args().collect::<Vec<String>>().get(0).unwrap().to_owned();

                // Check validity of TOTP code
                if HTTPServer::check_totp_code(&self.entry.get_password().expect(&crate::get_string("noTOTPSecret").format(&[binary_name])), &data.totp) {
                    // Get command & run it
                    let command: Vec<&str> = data.command.split_whitespace().collect();
                    let output = Command::new(&command[0])
                        .args(&command[1..])
                        .output()
                        .unwrap();

                    // Build result JSON & return it
                    let result: ResultData = ResultData { stdout: String::from_utf8_lossy(&output.stdout).to_string(), stderr: String::from_utf8_lossy(&output.stderr).to_string(), command_success: output.status.success(), success: true };
                    Response::text(serde_json::to_string(&result).unwrap())
                } else {
                    // Return invalid TOTP error
                    Response::text("{\"content\": \"Invalid TOTP code\", \"success\": false}")
                }
            } else {
                // Return method not allowed error
                Response::text("{\"content\": \"Method not allowed\", \"success\": false}")
            }
        });
    }

    fn check_totp_code(secret: &str, token: &str) -> bool {
        // Create TOTP object
        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 5, 
            Secret::Encoded(secret.to_string())
            .to_bytes().unwrap())
            .unwrap();
        
        // Check wether TOTP token is valid for now or last time interval
        (totp.generate_current().unwrap() == token) | (totp.generate(HTTPServer::get_unix_time() - totp.step) == token)
    }

    fn get_unix_time() -> u64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
} 