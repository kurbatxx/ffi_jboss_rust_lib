use crate::{
    create_string_pointer, login, APPDIR, COOKIE, JBOSS_FOLDER, LOGIN_DATA, PARSER_CLIENT, SITE_URL,
};

use chrono::{self, Duration};
use select::{
    document::Document,
    predicate::{Class, Name, Predicate},
};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::{ffi::CStr, fs};

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

        login::login(raw_login_data);
        return register_device(raw_register_json);
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

fn register_device_to_json(register_device_responce: RegisterDeviceResponce) -> Result<String> {
    let json = serde_json::to_string(&register_device_responce)?;
    Ok(json)
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
