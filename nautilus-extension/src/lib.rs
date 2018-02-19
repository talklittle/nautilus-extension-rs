extern crate gio_sys as gio_ffi;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate gtk_sys as gtk_ffi;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate nautilus_extension_sys as nautilus_ffi;

pub use column_provider::{Column, ColumnProvider};
pub use info_provider::{FileInfo, InfoProvider};
pub use menu_provider::{Menu, MenuItem, MenuProvider};
pub use nautilus_module::NautilusModule;
pub use property_page_provider::{PropertyPage, PropertyPageProvider};

pub mod column_provider;
pub mod info_provider;
pub mod menu_provider;
pub mod property_page_provider;
mod nautilus_module;
mod translate;

#[macro_export]
macro_rules! nautilus_module {
    ($register_fn:ident) => {
        static mut MODULE_TYPE_LIST: [GType; 1] = [0];

        #[no_mangle]
        pub extern "C" fn nautilus_module_initialize(module: *mut GTypeModule) {
            let module_type: GType = $register_fn(module);
            unsafe {
                MODULE_TYPE_LIST[0] = module_type;
            }
        }

        #[no_mangle]
        pub extern "C" fn nautilus_module_list_types(types: *mut *const GType, num_types: *mut c_int) {
            unsafe {
                *types = MODULE_TYPE_LIST.as_ptr();
                *num_types = MODULE_TYPE_LIST.len() as c_int;
            }
        }

        #[no_mangle]
        pub extern "C" fn nautilus_module_shutdown() {
        }
    }
}

#[macro_export]
macro_rules! nautilus_menu_item_activate_cb {
    ($extern_fn:ident, $safe_fn:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $extern_fn(_nautilusmenuitem: *mut GObject, user_data: gpointer) {
            use $crate::info_provider::FileInfo;
            use std::mem;

            let files: Box<Vec<FileInfo>> = Box::from_raw(mem::transmute(user_data));
            $safe_fn(*files);
        }

    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
