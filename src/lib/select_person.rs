use std::{ffi::CStr, fs};

use select::{document::Document, predicate::{Attr, Name}};
use serde::{Serialize, Deserialize};

use crate::{
    create_string_pointer, login, search::{ SchoolClient, FullName}, APPDIR, COOKIE, JBOSS_FOLDER, LOGIN_DATA, PARSER_CLIENT, SITE_URL,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CardStatus{
        card_id: String,
        is_active: bool,
        change_date: String
    }

#[no_mangle]
///# Safety
pub unsafe extern "C" fn select_person(raw_id: *const i8) -> *const i8 {
    let id_str = CStr::from_ptr(raw_id).to_str().unwrap();
    dbg!(id_str);

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

    if cookie.value().to_string() != COOKIE.read().unwrap().to_string() {
        println!("ffi: Перелогиниваюсь");

        let login_data_json_string = serde_json::to_string(&LOGIN_DATA).unwrap();
        let raw_login_data = create_string_pointer(login_data_json_string.as_str());

        login::login(raw_login_data);
        select_person(create_string_pointer(&id_str));
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
            "on", //"on",
        ),
        (
            //ID
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_12pc51",
            &id_str,
        ),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_18pc51",
            "-1",
        ),
        (
            //Фамилия
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_26pc51",
            "",
        ),
        (
            //Имя
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_30pc51",
            "",
        ),
        (
            //Отчество
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_34pc51",
            "",
        ),
        (
            //0 не важно наличе карт
            //1 есть карты
            //2 нет карт
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_43pc51",
            //&search_request.cards.to_string(),
            "0",
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

    let mut resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&search_param)
        .send()
        .unwrap();

    let mut resp_buf: Vec<u8> = vec![];
    resp.copy_to(&mut resp_buf)
        .expect("Копирование в буфер не удалось");
    let resp_text = String::from_utf8(resp_buf).unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "id.html",
        &resp_text,
    )
    .expect("Unable to write file");

    let select_param = [
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
            "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_635818149_108pc51",
            "j_id_jsp_635818149_109pc51",
        ),
        (   
            "workspaceSubView:workspaceForm", 
            "workspaceSubView:workspaceForm",
        ),
        ("javax.faces.ViewState", "j_id1"),
        (
            "workspaceSubView:workspaceForm:workspacePageSubView:clientListTable:0:j_id_jsp_635818149_64pc51", 
            "workspaceSubView:workspaceForm:workspacePageSubView:clientListTable:0:j_id_jsp_635818149_64pc51",
        ),
    ];

    let mut resp = PARSER_CLIENT
        .post(SITE_URL)
        .form(&select_param)
        .send()
        .unwrap();

    let mut resp_buf: Vec<u8> = vec![];
    resp.copy_to(&mut resp_buf)
        .expect("Копирование в буфер не удалось");
    let resp_text = String::from_utf8(resp_buf).unwrap();

    fs::write(
        APPDIR.to_owned() + "/" + JBOSS_FOLDER + "/" + "selected_by_id.html",
        &resp_text,
    )
    .expect("Unable to write file");


    let html_doc = Document::from(resp_text.as_str());
    
    let mut id_field = html_doc.find(Attr(
        "name",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_6pc51",
    ));
    let id = &id_field.next().unwrap().attr("value").unwrap_or("");
    dbg!(id);

    

    let mut name_field = html_doc.find(Attr(
        "name",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_26pc51",
    ));
    
    let name = &name_field.next().unwrap().attr("value").unwrap_or("");
    dbg!(name);

    let mut surname_field = html_doc.find(Attr(
        "name",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_24pc51",
    ));
    let surname = &surname_field.next().unwrap().attr("value").unwrap_or("");
    dbg!(surname);

    
    let mut patronymic_field = html_doc.find(Attr(
        "name",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_28pc51",
    ));
    let patronymic = &patronymic_field.next().unwrap().attr("value").unwrap_or("");
    dbg!(patronymic);


    let mut group_field = html_doc.find(Attr(
        "name",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_32pc51",
    ));
    let group = &group_field.next().unwrap().attr("value").unwrap_or("");
    dbg!(group);

    let mut balance_field = html_doc.find(Attr(
        "name",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_38pc51",
    ));
    let balance = &balance_field.next().unwrap().attr("value").unwrap_or("0");
    
    let balance = if balance.len() > 2  {
        &balance[0..balance.len() - 3]
    } else {
        "0"
    }.to_string();
    dbg!(&balance);



    let mut cards_header = html_doc.find(Attr(
        "id",
        "workspaceSubView:workspaceForm:workspacePageSubView:j_id_jsp_794730110_187pc51_header",
    ));
    let header_text = &cards_header.next().unwrap().inner_html().to_string();
    let card_value = header_text.chars().filter(|element| element.is_numeric()).collect::<String>().parse::<i32>().unwrap_or(0);
    dbg!(card_value);

    let mut cards_table = html_doc.find(Attr(
        "id",
        "workspaceSubView:workspaceForm:workspacePageSubView:clientCardTable:tb",
    ));

    

    
    let table = cards_table.next().unwrap();
    
    let rows  = table.find(Name("tr")).map(|item| {
        let card_id_node = item.children().nth(0);
        let card_id = if card_id_node.is_some(){
            card_id_node.unwrap().text()
        }else {
            "".to_string()
        };

        let card_status_node = item.children().nth(1);
        let card_status = if card_status_node.is_some(){
            if card_status_node.unwrap().text().contains("Выдана (активна)"){
                true 
            } else {
                false
            }
        } else {
            false
        };

        let card_date_node = item.children().nth(3);
        let card_date = if card_date_node.is_some(){
            card_date_node.unwrap().text()
        } else {
            "".to_string()
        };

        CardStatus{
            card_id,
            is_active: card_status,
            change_date: card_date,
        }
    }).collect::<Vec<_>>();
    dbg!(&rows);


    let client_info = SchoolClient{
        id: id.to_string(),
        full_name: FullName{
            name: name.to_string(),
            surname: surname.to_string(),
            patronymic: patronymic.to_string(),
        },
        group: group.to_string(),
        school: group.to_string(),
        balance: balance.to_string(),
        cards: rows,
    };

    let json = person_info_to_json(client_info);
    create_string_pointer(&json)
}


fn person_info_to_json(response: SchoolClient) -> String {
    let json = serde_json::to_string(&response).unwrap_or("{error: Не удалось получить данные о пользователе}".to_string());
    json
}