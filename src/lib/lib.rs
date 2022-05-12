pub mod initial;
pub mod login;
pub mod logout;
pub mod register_device;
pub mod search;

extern crate lazy_static;
extern crate reqwest;

use login::LoginRequest;

use std::ffi::CString;
use std::sync::RwLock;

const SITE_URL: &str = "https://bilim.integro.kz:8181/processor/back-office/index.faces";
const AUTH_URL: &str = "https://bilim.integro.kz:8181/processor/back-office/j_security_check";

const JBOSS_FOLDER: &str = "jboss";

static mut APPDIR: &str = "appdir";
static mut LOGIN_COUNTER: i32 = 0;

static mut LOGIN_DATA: LoginRequest = LoginRequest {
    login: "login",
    password: "password",
};

lazy_static::lazy_static! {
    static ref PARSER_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::builder()
        .use_native_tls()
        .cookie_store(true)
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    static ref COOKIE: RwLock<String> = RwLock::new("cookie".to_string());

}

pub fn create_string_pointer(string_to_ffi: &str) -> *const i8 {
    let c_string = CString::new(string_to_ffi).unwrap();
    let pointer = c_string.as_ptr();
    std::mem::forget(c_string);
    pointer
}
