use glib_ffi::{GList, g_list_length, g_list_nth_data};
use info_provider::FileInfo;
use nautilus_ffi::NautilusFileInfo;

pub fn file_info_vec_from_g_list(list: *mut GList) -> Vec<FileInfo> {
    let mut vec = Vec::new();
    unsafe {
        let length = g_list_length(list);
        for i in 0..length {
            let raw_file_info = g_list_nth_data(list, i) as *mut NautilusFileInfo;
            vec.push(FileInfo::new(raw_file_info));
        }
    }
    vec
}
