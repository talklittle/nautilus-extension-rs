use glib_ffi::{GList, g_list_append, gpointer};
use gobject_ffi::{GObject, g_signal_connect_data};
use gtk_ffi::GtkWidget;
use info_provider::FileInfo;
use libc::c_void;
use nautilus_ffi::{NautilusFileInfo, NautilusMenu, NautilusMenuItem, NautilusMenuProviderIface};
use nautilus_ffi::{nautilus_menu_new, nautilus_menu_append_item, nautilus_menu_item_new, nautilus_menu_item_set_submenu};
use std::borrow::Cow;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use translate::file_info_vec_from_g_list;

pub trait MenuProvider : Send + Sync {

    #[allow(unused_variables)]
    fn get_file_items(&self, window: *mut GtkWidget, files: &[FileInfo]) -> Vec<MenuItem> {
        Vec::new()
    }


    #[allow(unused_variables)]
    fn get_background_items(&self, window: *mut GtkWidget, current_folder: &FileInfo) -> Vec<MenuItem> {
        Vec::new()
    }

}

#[derive(Clone)]
pub struct Menu {
    menu_items: Vec<MenuItem>,
}

impl Menu {
    pub fn new(menu_items: &[MenuItem]) -> Menu {
        Menu {
            menu_items: menu_items.to_owned(),
        }
    }

    fn to_g_list(&self, files_user_data: *mut c_void) -> *mut GList {
        let mut raw_file_items: *mut GList = ptr::null_mut();

        for menu_item in &self.menu_items {
            let raw_name = CString::new(&menu_item.name as &str).unwrap().into_raw();
            let raw_label = CString::new(&menu_item.label as &str).unwrap().into_raw();
            let raw_tip = CString::new(&menu_item.tip as &str).unwrap().into_raw();
            let raw_icon =
                match menu_item.icon {
                    Some(ref ic) => CString::new(ic as &str).unwrap().into_raw(),
                    None => ptr::null_mut(),
                };

            unsafe {
                let raw_menuitem = nautilus_menu_item_new(raw_name, raw_label, raw_tip, raw_icon);
                raw_file_items = g_list_append(raw_file_items, raw_menuitem as *mut c_void);

                let submenu = &menu_item.submenu;
                match *submenu {
                    Some(ref submenu) => process_submenu(raw_menuitem, submenu, files_user_data),
                    None => (),
                }

                match menu_item.activate_fn {
                    Some(activate_fn) => connect_activate_signal(raw_menuitem, activate_fn, files_user_data),
                    None => (),
                }

                // deallocate CStrings
                let _ = CString::from_raw(raw_name);
                let _ = CString::from_raw(raw_label);
                let _ = CString::from_raw(raw_tip);
                if !raw_icon.is_null() {
                    let _ = CString::from_raw(raw_icon);
                }
            }
        }

        raw_file_items
    }

    fn to_raw(&self, files_user_data: *mut c_void) -> *mut NautilusMenu {
        let raw_menu = unsafe { nautilus_menu_new() };

        let menu_items = &self.menu_items;

        for menu_item in menu_items {
            let raw_name = CString::new(&menu_item.name as &str).unwrap().into_raw();
            let raw_label = CString::new(&menu_item.label as &str).unwrap().into_raw();
            let raw_tip = CString::new(&menu_item.tip as &str).unwrap().into_raw();
            let raw_icon =
                match menu_item.icon {
                    Some(ref icon) => CString::new(icon as &str).unwrap().into_raw(),
                    None => ptr::null_mut(),
                };

            unsafe {
                let raw_menuitem = nautilus_menu_item_new(raw_name, raw_label, raw_tip, raw_icon);
                nautilus_menu_append_item(raw_menu, raw_menuitem);

                let submenu = &menu_item.submenu;
                match *submenu {
                    Some(ref submenu) => process_submenu(raw_menuitem, submenu, files_user_data),
                    None => (),
                }

                match menu_item.activate_fn {
                    Some(activate_fn) => connect_activate_signal(raw_menuitem, activate_fn, files_user_data),
                    None => (),
                }

                // deallocate CStrings
                let _ = CString::from_raw(raw_name);
                let _ = CString::from_raw(raw_label);
                let _ = CString::from_raw(raw_tip);
                if !raw_icon.is_null() {
                    let _ = CString::from_raw(raw_icon);
                }
            }
        }

        raw_menu
    }
}

#[derive(Clone)]
pub struct MenuItem {
    name: Cow<'static, str>,
    label: Cow<'static, str>,
    tip: Cow<'static, str>,
    icon: Option<Cow<'static, str>>,
    submenu: Option<Menu>,
    activate_fn: Option<unsafe extern "C" fn(*mut GObject, gpointer)>,
}

impl MenuItem {
    pub fn new<S: Into<Cow<'static, str>>>(name: S, label: S, tip: S, icon: Option<S>) -> MenuItem {
        let icon = icon.map(|s| s.into());
        MenuItem {
            name: name.into(),
            label: label.into(),
            tip: tip.into(),
            icon,
            submenu: None,
            activate_fn: None,
        }
    }

    pub fn set_submenu(&mut self, submenu: &Menu) -> &mut MenuItem {
        self.submenu = Some(submenu.clone());
        self
    }

    pub fn set_activate_cb(&mut self, activate_cb: unsafe extern "C" fn(*mut GObject, gpointer)) -> &mut MenuItem {
        self.activate_fn = Some(activate_cb);
        self
    }
}

macro_rules! menu_provider_iface {
    ($iface_init_fn:ident, $get_file_items_fn:ident, $get_background_items_fn:ident, $rust_provider:ident, $set_rust_provider:ident) => {

        /// # Safety
        ///
        /// This generated function is used as a Nautilus callback. Do not call directly.
        /// Use `NautilusModule.add_menu_provider()` instead.
        #[no_mangle]
        pub unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            let iface_struct = iface as *mut NautilusMenuProviderIface;
            (*iface_struct).get_file_items = Some($get_file_items_fn);
            (*iface_struct).get_background_items = Some($get_background_items_fn);
        }

        #[no_mangle]
        pub extern "C" fn $get_file_items_fn(_provider: *mut c_void, window: *mut GtkWidget, files: *mut GList) -> *mut GList {
            if files.is_null() {
                return ptr::null_mut() as *mut GList;
            }

            let files_vec = file_info_vec_from_g_list(files);

            let file_items: Vec<MenuItem> =
                match *$rust_provider.lock().unwrap() {
                    Some(ref p) => p.get_file_items(window, &files_vec),
                    None => Vec::new(),
                };

            // dummy top-level Menu for easy recursion
            let top_menu = Menu {
                menu_items: file_items,
            };

            top_menu.to_g_list(Box::into_raw(Box::new(files_vec)) as *mut c_void)
        }

        #[no_mangle]
        pub extern "C" fn $get_background_items_fn(_provider: *mut c_void, window: *mut GtkWidget, current_folder: *mut NautilusFileInfo) -> *mut GList {
            if current_folder.is_null() {
                return ptr::null_mut() as *mut GList;
            }

            let file_info = FileInfo::new(current_folder);

            let file_items: Vec<MenuItem> =
                match *$rust_provider.lock().unwrap() {
                    Some(ref p) => p.get_background_items(window, &file_info),
                    None => Vec::new(),
                };

            // dummy top-level Menu for easy recursion
            let top_menu = Menu {
                menu_items: file_items,
            };

            top_menu.to_g_list(Box::into_raw(Box::new(file_info)) as *mut c_void)
        }

        pub fn $set_rust_provider(menu_provider: Box<dyn MenuProvider>) {
            *$rust_provider.lock().unwrap() = Some(menu_provider);
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Box<dyn MenuProvider>>> = Mutex::new(None);
        }
    }
}

fn process_submenu(raw_menuitem: *mut NautilusMenuItem, submenu: &Menu, files_user_data: *mut c_void) {
    let raw_submenu = submenu.to_raw(files_user_data);
    unsafe {
        nautilus_menu_item_set_submenu(raw_menuitem, raw_submenu);
    }
}

fn connect_activate_signal(raw_menuitem: *mut NautilusMenuItem, activate_fn: unsafe extern "C" fn(*mut GObject, gpointer), data: gpointer) {
    let activate_name = CString::new("activate").unwrap().into_raw();

    unsafe {
        g_signal_connect_data(
            raw_menuitem as *mut GObject,
            activate_name,
            Some(mem::transmute(activate_fn as *mut c_void)),
            data as *mut c_void,
            None,
            0
        );

        // deallocate CStrings
        let _ = CString::from_raw(activate_name);
    }
}

menu_provider_iface!(menu_provider_iface_init_0, menu_provider_get_file_items_0, menu_provider_get_background_items_0, MENU_PROVIDER_0, set_menu_provider_0);
menu_provider_iface!(menu_provider_iface_init_1, menu_provider_get_file_items_1, menu_provider_get_background_items_1, MENU_PROVIDER_1, set_menu_provider_1);
menu_provider_iface!(menu_provider_iface_init_2, menu_provider_get_file_items_2, menu_provider_get_background_items_2, MENU_PROVIDER_2, set_menu_provider_2);
menu_provider_iface!(menu_provider_iface_init_3, menu_provider_get_file_items_3, menu_provider_get_background_items_3, MENU_PROVIDER_3, set_menu_provider_3);
menu_provider_iface!(menu_provider_iface_init_4, menu_provider_get_file_items_4, menu_provider_get_background_items_4, MENU_PROVIDER_4, set_menu_provider_4);
menu_provider_iface!(menu_provider_iface_init_5, menu_provider_get_file_items_5, menu_provider_get_background_items_5, MENU_PROVIDER_5, set_menu_provider_5);
menu_provider_iface!(menu_provider_iface_init_6, menu_provider_get_file_items_6, menu_provider_get_background_items_6, MENU_PROVIDER_6, set_menu_provider_6);
menu_provider_iface!(menu_provider_iface_init_7, menu_provider_get_file_items_7, menu_provider_get_background_items_7, MENU_PROVIDER_7, set_menu_provider_7);
menu_provider_iface!(menu_provider_iface_init_8, menu_provider_get_file_items_8, menu_provider_get_background_items_8, MENU_PROVIDER_8, set_menu_provider_8);
menu_provider_iface!(menu_provider_iface_init_9, menu_provider_get_file_items_9, menu_provider_get_background_items_9, MENU_PROVIDER_9, set_menu_provider_9);

pub fn menu_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        menu_provider_iface_init_0,
        menu_provider_iface_init_1,
        menu_provider_iface_init_2,
        menu_provider_iface_init_3,
        menu_provider_iface_init_4,
        menu_provider_iface_init_5,
        menu_provider_iface_init_6,
        menu_provider_iface_init_7,
        menu_provider_iface_init_8,
        menu_provider_iface_init_9,
    ]
}

pub fn rust_menu_provider_setters() -> Vec<fn(Box<dyn MenuProvider>)> {
    vec![
        set_menu_provider_0,
        set_menu_provider_1,
        set_menu_provider_2,
        set_menu_provider_3,
        set_menu_provider_4,
        set_menu_provider_5,
        set_menu_provider_6,
        set_menu_provider_7,
        set_menu_provider_8,
        set_menu_provider_9,
    ]
}

static NEXT_MENU_PROVIDER_IFACE_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn take_next_menu_provider_iface_index() -> usize {
    NEXT_MENU_PROVIDER_IFACE_INDEX.fetch_add(1, Ordering::SeqCst)
}
