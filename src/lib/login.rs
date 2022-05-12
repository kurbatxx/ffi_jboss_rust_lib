use std::{ffi::CStr, fs};

use select::{document::Document, predicate::Attr};
use serde::{Deserialize, Serialize};
use serde_json::Result;

use crate::{
    create_string_pointer, APPDIR, AUTH_URL, COOKIE, JBOSS_FOLDER, LOGIN_COUNTER, LOGIN_DATA,
    PARSER_CLIENT, SITE_URL,
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LoginRequest<'a> {
    pub(crate) login: &'a str,
    pub(crate) password: &'a str,
}

#[derive(Debug, Default, Serialize)]
pub struct LoginResponse {
    cookie: String,
    error: String,
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn login(raw_login_data: *const i8) -> *const i8 {
    let raw_login_data = CStr::from_ptr(raw_login_data).to_str().unwrap();
    println!("{}", raw_login_data);

    let login_data: LoginRequest = serde_json::from_str(&raw_login_data).unwrap();

    let _ = PARSER_CLIENT.get(SITE_URL).send().unwrap();

    let auth_params = [
        ("j_username", &login_data.login),
        ("j_password", &login_data.password),
    ];
    let mut resp = PARSER_CLIENT
        .post(AUTH_URL)
        .form(&auth_params)
        .send()
        .unwrap();

    let mut resp_buf: Vec<u8> = vec![];
    resp.copy_to(&mut resp_buf)
        .expect("Копирование в буфер не удалось");
    let resp_text = String::from_utf8(resp_buf).unwrap();

    let cookie_raw = &resp.cookies().next().unwrap();
    let cookie = cookie_raw.value();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "login.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let doc_html = Document::from(resp_text.as_str());
    let auth_check = doc_html.find(Attr("id", "headerForm:sysuser")).next();

    let mut eror_message = "";
    if auth_check != None {
        println!("ffi: вошел");
        LOGIN_COUNTER = 0;

        let mut true_cookie = COOKIE.write().unwrap();
        *true_cookie = cookie.to_string();

        LOGIN_DATA = login_data;
    } else {
        println!("ffi: НЕ вошел");
        LOGIN_COUNTER = LOGIN_COUNTER + 1;

        eror_message = "Неправильный логин или пароль";
    }
    let json = authorization_token_to_json(LoginResponse {
        cookie: COOKIE.read().unwrap().to_string(),
        error: eror_message.to_string(),
    })
    .expect("Не удалось создать JSON");

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "login.json",
        &json,
    )
    .expect("Unable to write file");

    create_string_pointer(json.as_str())
}

fn authorization_token_to_json(authorization_token: LoginResponse) -> Result<String> {
    let json = serde_json::to_string(&authorization_token)?;
    Ok(json)
}
