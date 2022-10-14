use std::fs;

use crate::{APPDIR, JBOSS_FOLDER, PARSER_CLIENT, SITE_URL};

#[no_mangle]
///# Safety
pub unsafe extern "C" fn logout() {
    let logout_params = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        ("headerForm", "headerForm"),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "headerForm:j_id_jsp_659141934_66",
            "headerForm:j_id_jsp_659141934_66",
        ),
    ];

    let _ = PARSER_CLIENT
        .post(SITE_URL)
        .form(&logout_params)
        .send()
        .unwrap();

    let mut resp = PARSER_CLIENT.get(SITE_URL).send().unwrap();

    let mut resp_buf: Vec<u8> = vec![];
    resp.copy_to(&mut resp_buf)
        .expect("Копирование в буфер не удалось");
    let resp_text = String::from_utf8(resp_buf).unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "logout.html",
        &resp_text,
    )
    .expect("Unable to write file");
    println!("ffi: ВЫШЕЛ");
}
