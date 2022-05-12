use std::{ffi::CStr, fs};

use crate::{APPDIR, JBOSS_FOLDER};

#[no_mangle]
///# Safety
pub unsafe extern "C" fn initial(raw_appdir: *const i8) {
    APPDIR = CStr::from_ptr(raw_appdir).to_str().unwrap();
    fs::create_dir_all(APPDIR.to_owned() + "/" + JBOSS_FOLDER)
        .expect("Не удалось создать директорию");
}
