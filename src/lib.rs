extern crate lazy_static;
extern crate reqwest;
use std::env;

use select::node::Node;
use serde::{Deserialize, Serialize};
use serde_json::{json, Result};

use select::document::Document;
use select::predicate::{Attr, Name, Predicate};

use std::{
    ffi::{CStr, CString},
    fs,
};

const SITE_URL: &str = "https://bilim.integro.kz:8181/processor/back-office/index.faces";
const AUTH_URL: &str = "https://bilim.integro.kz:8181/processor/back-office/j_security_check";

const JBOSS_FOLDER: &str = "jboss";

#[macro_use]
lazy_static::lazy_static! {
    static ref PARSER_CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::builder()
        .use_native_tls()
        .cookie_store(true)
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
}

static mut USERNAME: &str = "username";
static mut PASSWORD: &str = "password";
static mut APPDIR: &str = "appdir";


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SchoolClient {
    id: String,
    fullname: FullName,
    group: String,
    school: String,
    balance: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullName {
    name: String,
    surname: String,
    patronymic: String,
}

#[derive(Debug, Default, Serialize)]
pub struct AuthorizationToken {
    cookie: String,
    error: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SearchRequest {
    id: i32,
    request: String,
    school_id: i32,
    cards: i32,
    page: i32,
    show_delete: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SearchResponse {
    clients: Vec<SchoolClient>,
    all_pages: i32,
    error: String,
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn initial(raw_appdir: *const i8) {
    APPDIR = CStr::from_ptr(raw_appdir).to_str().unwrap();
    fs::create_dir_all(APPDIR.to_owned() + "/" + JBOSS_FOLDER);
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn login(raw_username: *const i8, raw_password: *const i8) -> *const i8 {
    let username = CStr::from_ptr(raw_username).to_str().unwrap();
    let password = CStr::from_ptr(raw_password).to_str().unwrap();

    let _ = PARSER_CLIENT.get(SITE_URL).send().unwrap();

    let auth_params = [("j_username", username), ("j_password", password)];
    let mut resp = PARSER_CLIENT
        .post(AUTH_URL)
        .form(&auth_params)
        .send()
        .unwrap();

    let mut resp_buf: Vec<u8> = vec![];
    resp.copy_to(&mut resp_buf)
        .expect("Копирование в буфер не удалось");
    let html_text = String::from_utf8(resp_buf).unwrap();

    let cookie_raw = &resp.cookies().next().unwrap();
    let cookie = cookie_raw.value();

    fs::write(APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "login.html", &html_text)
        .expect("Unable to write file");

    let doc_html = Document::from(html_text.as_str());
    let auth_check = doc_html.find(Attr("id", "headerForm:sysuser")).next();

    if auth_check != None {
        println!("Вошел..");
        let json = authorization_token_to_json(&AuthorizationToken {
            cookie: cookie.to_string(),
            error: "".to_string(),
        })
        .expect("Не удалось создать JSON");
        create_string_pointer(json.as_str())
    } else {
        println!("НЕ Вошел....");
        let json = authorization_token_to_json(&AuthorizationToken {
            cookie: cookie.to_string(),
            error: "Неправильный логин или пароль".to_string(),
        })
        .expect("Не удалось создать JSON");
        create_string_pointer(json.as_str())
    }
}

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

    let mut resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&logout_params)
        .send()
        .unwrap();

    resp = PARSER_CLIENT.get(SITE_URL).send().unwrap();

    let mut resp_buf: Vec<u8> = vec![];
    resp.copy_to(&mut resp_buf)
        .expect("Копирование в буфер не удалось");
    let html_text = String::from_utf8(resp_buf).unwrap();

    let cookie_raw = &resp.cookies().next().unwrap();
    let cookie = cookie_raw.value();

    fs::write(APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "logout.html", &html_text)
        .expect("Unable to write file");
    println!("ВЫШЕЛ--");
}

fn create_string_pointer(string_to_dart: &str) -> *const i8 {
    let c_string = CString::new(string_to_dart).unwrap();
    let pointer = c_string.as_ptr();
    std::mem::forget(c_string);
    pointer
}

fn vector_clients_to_json(response: SearchResponse) -> Result<String> {
    let json = serde_json::to_string(&response)?;
    Ok(json)
}


fn authorization_token_to_json(authorization_token: &AuthorizationToken) -> Result<String> {
    let json = serde_json::to_string(authorization_token)?;
    Ok(json)
}

fn client_amount(html_search_result: select::document::Document) -> i32 {
    let mut client_amout = html_search_result.find(Attr(
        "id",
        "workspaceSubView:workspaceForm:workspaceTogglePanel_header",
    ));

    let all_text = client_amout.next().unwrap().inner_html();
    let digit_text: String = all_text.chars().filter(|c| c.is_digit(10)).collect();
    dbg!(&digit_text);
    digit_text.parse::<i32>().unwrap()
}

fn calculate_pages(client_amount: i32) -> i32 {
    if client_amount == 0 {
        0
    } else if client_amount % 20 == 0 {
        client_amount / 20
    } else {
        client_amount / 20 + 1
    }
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn search_person(raw_search_json: *const i8) -> *const i8 {
    let search_json = CStr::from_ptr(raw_search_json).to_str().unwrap();
    let search_request: SearchRequest = serde_json::from_str(search_json).unwrap();
    println!("{}", search_request.request);

    let fio = search_request.request.as_str();
    let fullname = get_fio(fio.to_string());

    let list_client_params = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "mainMenuSubView:mainMenuForm:mainMenuselectedItemName",
            "showClientListMenuItem",
        ),
        (
            "panelMenuStatemainMenuSubView:mainMenuForm:clientGroupMenu",
            "opened",
        ),
        (
            "panelMenuActionmainMenuSubView:mainMenuForm:showClientListMenuItem",
            "mainMenuSubView:mainMenuForm:showClientListMenuItem",
        ),
        (
            "mainMenuSubView:mainMenuForm",
            "mainMenuSubView:mainMenuForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "mainMenuSubView:mainMenuForm:showClientListMenuItem",
            "mainMenuSubView:mainMenuForm:showClientListMenuItem",
        ),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&list_client_params)
        .send()
        .unwrap();
    //fs::write("list.html", &resp.text().unwrap()).expect("Unable to write file");

    let cookie_in = &resp.cookies().next().unwrap();
    dbg!(&cookie_in.value());

    let search_param = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_1pc51",
            "true",
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_8pc51",
            "on",
        ),
        (
            //ID
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_12pc51",
            "",
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_18pc51",
            "-1",
        ),
        (
            //Фамилия
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_26pc51",
            fullname.surname.as_str(),
        ),
        (
            //Имя
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_30pc51",
            fullname.name.as_str(),
        ),
        (
            //Отчество
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_34pc51",
            fullname.patronymic.as_str(),
        ),
        (
            //0 не важно наличе карт
            //1 есть карты
            //2 нет карт
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_43pc51",
            &search_request.cards.to_string(),
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_46pc51",
            "0",
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_108pc51",
            "j_id_jsp_635818149_109pc51",
        ),
        (
            "workspaceSubView:workspaceForm",
            "workspaceSubView:workspaceForm",
        ),
        ("javax.faces.ViewState", "j_id1"),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_53pc51",
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_53pc51",
        ),
    ];
    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&search_param)
        .send()
        .unwrap();
    let resp_text = &resp.text().unwrap();

    fs::write(APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "search.html", &resp_text)
       .expect("Unable to write file");

    let html_search_result = Document::from(resp_text.as_str());
    let client_amount = client_amount(html_search_result);
    let pages: i32 = calculate_pages(client_amount);
    println!("Всего {} страниц", pages);

    let mut result_vector = get_person_data(resp_text);
    let  current_page = search_request.page;


    if current_page == 0 {
        for page_index in 2..pages + 1 {
            select_current_page(pages, &mut result_vector, page_index)
        }
    } else if current_page == 1 {
    } else if current_page == 2 || current_page <= current_page{
        result_vector = Vec::new();
        select_current_page(pages, &mut result_vector, current_page)
    } else {
    }

    let search_response = SearchResponse {
        clients: result_vector,
        all_pages: pages,
        error: "".parse().unwrap()
    };

    let json = vector_clients_to_json(search_response).expect("Не удалось создать JSON");
    fs::write(APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "json_result.json", &json)
        .expect("Unable to write file");

    //Для FFI
    let string_to_dart = CString::new(json).unwrap();
    let pointer = string_to_dart.as_ptr();
    std::mem::forget(string_to_dart);
    pointer
}

fn select_current_page(pages: i32, result_vector: &mut Vec<SchoolClient>, page_index: i32) {
    let search_param_next = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        ("javax.faces.ViewState", "j_id1"),
        ("workspaceSubView:workspaceForm:workspacePageSubView:clientListTable:j_id_jsp_635818149_104pc51", &page_index.to_string()),
        ("ajaxSingle", "workspaceSubView:workspaceForm:workspacePageSubView:clientListTable:j_id_jsp_635818149_104pc51"),
        ("AJAX:EVENTS_COUNT", "1"),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&search_param_next)
        .send()
        .unwrap();

    let resp_text = &resp.text().unwrap();
    result_vector.append(&mut get_person_data(resp_text));

    if page_index == pages {
        dbg!(&result_vector.len());
        // fs::write(
        //     JBOSS_FOLDER.to_owned() + "/" + "search_next.html",
        //     &resp_text,
        // )
        // .expect("Unable to write file");
    }
}

fn get_person_data(resp_text: &str) -> Vec<SchoolClient> {
    let document = Document::from(resp_text);
    let mut on_page_clients = Vec::new();
    for node in document.find(
        Attr(
            "id",
            "workspaceSubView:workspaceForm:workspacePageSubView:clientListTable:tb",
        )
        .descendant(Name("tr")),
    ) {
        let cells: Vec<Node> = node.find(Name("td")).collect();
        let fullname = get_fio(cells[3].text());

        let client = SchoolClient {
            id: cells[1].text(),
            fullname: fullname,
            group: cells[4].text(),
            school: cells[6].text(),
            //delete 3 symbols ",00"
            balance: cells[7].text()[0..cells[7].text().len() - 3].to_string(),
        };

        on_page_clients.push(client);
    }
    on_page_clients
}

fn get_fio(fio: String) -> FullName {
    let mut chunks: Vec<_> = fio.split_whitespace().map(|s| s.to_string()).collect();

    match chunks.len() {
        0 => {
            for _ in 1..=3 {
                chunks.push("".to_string());
            }
        }
        1 => {
            for _ in 2..=3 {
                chunks.push("".to_string());
            }
        }
        2 => chunks.push("".to_string()),
        3 => (),
        _ => {
            let patronymic_vec = chunks.split_off(2);
            let patronymic = patronymic_vec.join(" ");
            chunks.push(patronymic);
        }
    }
    FullName {
        surname: chunks[0].to_string(),
        name: chunks[1].to_string(),
        patronymic: chunks[2].to_string(),
    }
}
