use jboss::{create_string_pointer, initial, login};

fn main() {
    let curdir = std::env::current_dir().expect("Не удалось получить директорию запуска программы");
    let curdir_str = curdir
        .to_str()
        .expect("Не удалось пребразовать каталог в строку");
    let raw_appdir = create_string_pointer(curdir_str);

    let raw_username = create_string_pointer("");
    let raw_password = create_string_pointer("");
    unsafe {
        initial(raw_appdir);
        login(raw_username, raw_password);
    }
}
