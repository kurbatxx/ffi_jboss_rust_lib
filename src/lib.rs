extern crate lazy_static;
extern crate reqwest;

use chrono::{self, Duration};

use select::node::Node;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};

use std::sync::RwLock;
use std::{
    ffi::{CStr, CString},
    fs,
};

const SITE_URL: &str = "https://bilim.integro.kz:8181/processor/back-office/index.faces";
const AUTH_URL: &str = "https://bilim.integro.kz:8181/processor/back-office/j_security_check";

const JBOSS_FOLDER: &str = "jboss";
static mut APPDIR: &str = "appdir";
static mut LOGIN_COUNTER: i32 = 0;

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

static mut LOGIN_DATA: LoginData = LoginData {
    username: "login",
    password: "password",
};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LoginData<'a> {
    username: &'a str,
    password: &'a str,
}

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

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct RegisterDeviceRequest {
    client_id: u32,
    rfid_id: u32,
    device_id: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegisterDeviceResponce {
    original_message: String,
    result_message: String,
    client: String,
    register: bool,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SelectedClient {
    selected: bool,
    name: String,
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn initial(raw_appdir: *const i8) {
    APPDIR = CStr::from_ptr(raw_appdir).to_str().unwrap();
    fs::create_dir_all(APPDIR.to_owned() + "/" + JBOSS_FOLDER)
        .expect("Не удалось создать директорию");
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn login(raw_login_data: *const i8) -> *const i8 {
    let raw_login_data = CStr::from_ptr(raw_login_data).to_str().unwrap();
    let login_data: LoginData = serde_json::from_str(&raw_login_data).unwrap();

    let _ = PARSER_CLIENT.get(SITE_URL).send().unwrap();

    let auth_params = [
        ("j_username", &login_data.username),
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
    let json = authorization_token_to_json(AuthorizationToken {
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

    //let cookie_raw = &resp.cookies().next().unwrap();
    //let cookie = cookie_raw.value();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "logout.html",
        &resp_text,
    )
    .expect("Unable to write file");
    println!("ffi: ВЫШЕЛ");
}

#[no_mangle]
///# Safety
pub unsafe extern "C" fn search_person(raw_search_json: *const i8) -> *const i8 {
    let search_json = CStr::from_ptr(raw_search_json).to_str().unwrap();
    let search_request: SearchRequest = serde_json::from_str(search_json).unwrap();
    println!("{}", search_request.request);

    let fio = search_request.request.as_str();
    let fullname = get_fio(fio.to_string());
    let show_delete = search_request.show_delete;

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

    let cookie = &resp.cookies().next().unwrap();
    dbg!(&cookie.value());

    if cookie.value().to_string() != COOKIE.read().unwrap().to_string() {
        println!("ffi: Перелогиниваюсь");

        let login_data_json_string = serde_json::to_string(&LOGIN_DATA).unwrap();
        let raw_login_data = create_string_pointer(login_data_json_string.as_str());

        login(raw_login_data);
        search_person(raw_search_json);
    }

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
            //Показывать удалённых
            "workspaceSubView:workspaceForm:workspacePageSubView:showDeletedClients",
            if show_delete { "on" } else { "" }, //"on",
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

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "search.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let html_search_result = Document::from(resp_text.as_str());
    let client_amount = client_amount(html_search_result);
    let pages: i32 = calculate_pages(client_amount);
    println!("Всего {} страниц", pages);

    let mut result_vector = get_person_data(resp_text);
    let current_page = search_request.page;

    if current_page == 0 {
        for page_index in 2..pages + 1 {
            select_current_page(pages, &mut result_vector, page_index)
        }
    } else if current_page == 2 || current_page <= current_page {
        result_vector = Vec::new();
        select_current_page(pages, &mut result_vector, current_page)
    }

    let search_response = SearchResponse {
        clients: result_vector,
        all_pages: pages,
        error: "".parse().unwrap(),
    };

    let json = vector_clients_to_json(search_response).expect("Не удалось создать JSON");
    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "json_result.json",
        &json,
    )
    .expect("Unable to write file");

    create_string_pointer(&json)
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

#[no_mangle]
///# Safety
pub unsafe extern "C" fn register_device(raw_register_json: *const i8) -> *const i8 {
    let register_json = CStr::from_ptr(raw_register_json).to_str().unwrap();
    let register_request: RegisterDeviceRequest = serde_json::from_str(register_json).unwrap();
    dbg!(register_request);

    let cards_register_params = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "mainMenuSubView:mainMenuForm:mainMenuselectedItemName",
            "createCardMenuItem",
        ),
        (
            "panelMenuStatemainMenuSubView:mainMenuForm:cardGroupMenu",
            "opened",
        ),
        (
            "panelMenuActionmainMenuSubView:mainMenuForm:createCardMenuItem",
            "mainMenuSubView:mainMenuForm:createCardMenuItem",
        ),
        (
            "mainMenuSubView:mainMenuForm",
            "mainMenuSubView:mainMenuForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "mainMenuSubView:mainMenuForm:createCardMenuItem",
            "mainMenuSubView:mainMenuForm:createCardMenuItem",
        ),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&cards_register_params)
        .send()
        .unwrap();

    let cookie = &resp.cookies().next().unwrap();
    dbg!(&cookie.value());

    if cookie.value().to_string() != COOKIE.read().unwrap().to_string() {
        println!("ffi: Перелогиниваюсь");

        let login_data_json_string = serde_json::to_string(&LOGIN_DATA).unwrap();
        let raw_login_data = create_string_pointer(login_data_json_string.as_str());

        login(raw_login_data);
        search_person(raw_register_json);
    }

    let resp_text = &resp.text().unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "cards.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let client_select_param = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "workspaceSubView:workspaceForm",
            "workspaceSubView:workspaceForm",
        ),
        (
            "clientSelectSubView:modalClientSelectorForm",
            "clientSelectSubView:modalClientSelectorForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_4pc51",
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_4pc51",
        ),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&client_select_param)
        .send()
        .unwrap();

    let resp_text = &resp.text().unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "cards_select_client.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let filter_param = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "clientSelectSubView:modalClientSelectorForm:modalClientSelectorFilterPanel",
            "true",
        ),
        (
            "clientSelectSubView:modalClientSelectorForm:j_id_jsp_1535611719_13pc27",
            &register_request.client_id.to_string()[..],
        ),
        (
            "clientSelectSubView:modalClientSelectorForm",
            "clientSelectSubView:modalClientSelectorForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "clientSelectSubView:modalClientSelectorForm:j_id_jsp_1535611719_21pc27",
            "clientSelectSubView:modalClientSelectorForm:j_id_jsp_1535611719_21pc27",
        ),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&filter_param)
        .send()
        .unwrap();

    let resp_text = &resp.text().unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "filter.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let select_first_client = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "clientSelectSubView:modalClientSelectorForm:modalClientSelectorFilterPanel",
            "true",
        ),
        (
            "clientSelectSubView:modalClientSelectorForm:j_id_jsp_1535611719_13pc27",
            &register_request.client_id.to_string()[..],
        ),
        (
            "clientSelectSubView:modalClientSelectorForm",
            "clientSelectSubView:modalClientSelectorForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "clientSelectSubView:modalClientSelectorForm:modalClientSelectorTable:0:j_id_jsp_1535611719_24pc27",
            "clientSelectSubView:modalClientSelectorForm:modalClientSelectorTable:0:j_id_jsp_1535611719_24pc27",
        ),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&select_first_client)
        .send()
        .unwrap();

    let resp_text = &resp.text().unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "select_first_client.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let select_client = get_select_name(resp_text);

    let sumbit_client = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "clientSelectSubView:modalClientSelectorForm",
            "clientSelectSubView:modalClientSelectorForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "clientSelectSubView:modalClientSelectorForm:j_id_jsp_1535611719_41pc27",
            "clientSelectSubView:modalClientSelectorForm:j_id_jsp_1535611719_41pc27",
        ),
    ];

    let resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&sumbit_client)
        .send()
        .unwrap();

    let resp_text = &resp.text().unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "sumbit_client.html",
        &resp_text,
    )
    .expect("Unable to write file");

    //// Регистрация карты
    let now = chrono::offset::Local::now();
    let now10year = now + Duration::days(365 * 10);

    let now_full = now.format("%d.%m.%Y").to_string();
    let now_split = now.format("%m/%Y").to_string();

    let now10year_full = now10year.format("%d.%m.%Y").to_string();
    let now10year_split = now10year.format("%m/%Y").to_string();

    println!("UTC now in a custom format is: {}", now_full);
    println!("UTC now in a custom format is: {}", now_split);

    let sumbit_client = [
        ("AJAXREQUEST", "j_id_jsp_659141934_0"),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_6pc51",
            &register_request.rfid_id.to_string()[..],
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_8pc51",
            &register_request.client_id.to_string()[..],
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_10pc51",
            &register_request.device_id.to_string()[..],
        ),
        ("workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_13pc51InputDate", now_full.as_str()),
        ("workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_13pc51InputCurrentDate", now_split.as_str()),
        ("workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_15pc51InputDate", now10year_full.as_str()),
        ("workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_15pc51InputCurrentDate", now10year_split.as_str()),
        ("workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_17pc51", "0"),
        ("workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_20pc51", "1"),
        (
            "workspaceSubView:workspaceForm",
            "workspaceSubView:workspaceForm",
        ),
        ("autoScroll", ""),
        ("javax.faces.ViewState", "j_id1"),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_23pc51",
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_202606668_23pc51",
        ),
    ];

    let register_card_json: String;
    if select_client.selected {
        let resp = PARSER_CLIENT
            .post(SITE_URL)
            .form(&sumbit_client)
            .send()
            .unwrap();

        let resp_text = &resp.text().unwrap();

        fs::write(
            APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "final_register_card.html",
            &resp_text,
        )
        .expect("Unable to write file");

        let message = get_register_message(resp_text);
        let message = message.as_str();
        println!("{}", message);

        let register: bool;
        let mut result_message: &str;
        match message {
            "Карта зарегистрирована успешно" => {
                result_message = "Устройство зарегистрировано";
                register = true;
            }
            "Ошибка при регистрации карты: Данный клиент имеет незаблокированную(ые) карту(ы)." =>
            {
                result_message = "У пользователя есть незаблокированное устройство";
                register = false;
            }

            _ => {
                result_message = "Неизвестная ошибка";
                register = false;
            }
        }

        let res_msg = format!(
            "Устройство {} уже зарегистрировано",
            register_request.rfid_id.to_string()
        );

        if !register {
            let card_error_message = format!(
                "Ошибка при регистрации карты: Карта {} уже зарегистрирована",
                register_request.rfid_id.to_string()
            );

            if card_error_message == message {
                result_message = res_msg.as_str();
            }
        }

        register_card_json = register_device_to_json(RegisterDeviceResponce {
            original_message: message.to_string(),
            result_message: result_message.to_string(),
            client: select_client.name,
            register: register,
        })
        .expect("Не удалось создать json");
    } else {
        register_card_json = register_device_to_json(RegisterDeviceResponce {
            original_message: "".to_string(),
            result_message: "Не выбран пользователь".to_string(),
            client: "".to_string(),
            register: false,
        })
        .expect("Не удалось создать json");
    }

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "register_card_json.json",
        &register_card_json,
    )
    .expect("Unable to write file");

    create_string_pointer(&register_card_json)
}

fn get_select_name(resp_text: &str) -> SelectedClient {
    let document = Document::from(resp_text);
    let node = document
        .find(Class("borderless-grid").descendant(Name("input")))
        .take(1);

    let mut select_name = "".to_string();
    let mut selected = false;

    for i in node {
        select_name = i.attr("value").unwrap().to_string();
        selected = true;
    }
    SelectedClient {
        selected: selected,
        name: select_name,
    }
}

fn get_register_message(resp_text: &str) -> String {
    let document = Document::from(resp_text);
    let mut node = document.find(Class("rich-messages-label")).take(1);
    node.next().unwrap().text()
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

pub fn create_string_pointer(string_to_ffi: &str) -> *const i8 {
    let c_string = CString::new(string_to_ffi).unwrap();
    let pointer = c_string.as_ptr();
    std::mem::forget(c_string);
    pointer
}

fn vector_clients_to_json(response: SearchResponse) -> Result<String> {
    let json = serde_json::to_string(&response)?;
    Ok(json)
}

fn authorization_token_to_json(authorization_token: AuthorizationToken) -> Result<String> {
    let json = serde_json::to_string(&authorization_token)?;
    Ok(json)
}

fn register_device_to_json(register_device_responce: RegisterDeviceResponce) -> Result<String> {
    let json = serde_json::to_string(&register_device_responce)?;
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
