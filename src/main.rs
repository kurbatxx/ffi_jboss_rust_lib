use jboss::{create_string_pointer, initial, login, register_device};

fn main() {
    let curdir = std::env::current_dir().expect("Не удалось получить директорию запуска программы");
    let curdir_str = curdir
        .to_str()
        .expect("Не удалось пребразовать каталог в строку");
    let raw_appdir = create_string_pointer(curdir_str);

    let raw_username = create_string_pointer("");
    let raw_password = create_string_pointer("");

    let raw_register_json = create_string_pointer(
        "{\"client_id\": 85800142, \"rfid_id\": 1735547725, \"device_id\": 1}",
    );

    unsafe {
        initial(raw_appdir);
        login(raw_username, raw_password);
        register_device(raw_register_json);
    }
}
