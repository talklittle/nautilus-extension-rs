use glib_ffi::{GList, g_list_append, gpointer};
use gtk_ffi::GtkWidget;
use info_provider::FileInfo;
use libc::c_void;
use nautilus_ffi::{NautilusPropertyPageProvider, nautilus_property_page_new};
use std::borrow::Cow;
use std::ffi::CString;
use std::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use translate::file_info_vec_from_g_list;

pub struct PropertyPage {
    pub name: Cow<'static, str>,
    pub raw_label: *mut GtkWidget,
    pub raw_page: *mut GtkWidget,
}

impl PropertyPage {
    pub fn new<S: Into<Cow<'static, str>>>(name: S, raw_label: *mut GtkWidget, raw_page: *mut GtkWidget) -> PropertyPage {
        PropertyPage {
            name: name.into(),
            raw_label,
            raw_page,
        }
    }
}

pub trait PropertyPageProvider : Send {
    fn get_pages(&self, files: &Vec<FileInfo>) -> Vec<PropertyPage>;
}

macro_rules! property_page_provider_iface {
    ($iface_init_fn:ident, $get_pages_fn:ident, $rust_provider:ident, $set_rust_provider:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            use nautilus_ffi::NautilusPropertyPageProviderIface;

            let iface_struct = iface as *mut NautilusPropertyPageProviderIface;
            (*iface_struct).get_pages = Some($get_pages_fn);
        }

        #[no_mangle]
        pub extern "C" fn $get_pages_fn(_provider: *mut NautilusPropertyPageProvider, raw_files: *mut GList) -> *mut GList {
            let mut pages_g_list = ptr::null_mut();

            let pages = match *$rust_provider.lock().unwrap() {
                Some(ref p) => p.get_pages(&file_info_vec_from_g_list(raw_files)),
                None => Vec::new(),
            };

            for page in pages {
                let name = CString::new(&page.name as &str).unwrap().into_raw();
                let label = page.raw_label;
                let page_widget = page.raw_page;

                unsafe {
                    let page_c = nautilus_property_page_new(name, label, page_widget);
                    pages_g_list = g_list_append(pages_g_list, page_c as *mut c_void);

                    // deallocate CStrings
                    let _ = CString::from_raw(name);
                }
            }

            pages_g_list
        }

        pub fn $set_rust_provider(page_provider: Box<dyn PropertyPageProvider>) {
            *$rust_provider.lock().unwrap() = Some(page_provider);
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Box<dyn PropertyPageProvider>>> = Mutex::new(None);
        }
    }
}

// Let library consumer add up to 10 ColumnProviders, should be more than enough. Each has its own Vec of columns.
property_page_provider_iface!(property_page_provider_iface_init_0, property_page_provider_get_pages_0, PROPERTY_PAGE_PROVIDER_0, set_property_page_provider_0);
property_page_provider_iface!(property_page_provider_iface_init_1, property_page_provider_get_pages_1, PROPERTY_PAGE_PROVIDER_1, set_property_page_provider_1);
property_page_provider_iface!(property_page_provider_iface_init_2, property_page_provider_get_pages_2, PROPERTY_PAGE_PROVIDER_2, set_property_page_provider_2);
property_page_provider_iface!(property_page_provider_iface_init_3, property_page_provider_get_pages_3, PROPERTY_PAGE_PROVIDER_3, set_property_page_provider_3);
property_page_provider_iface!(property_page_provider_iface_init_4, property_page_provider_get_pages_4, PROPERTY_PAGE_PROVIDER_4, set_property_page_provider_4);
property_page_provider_iface!(property_page_provider_iface_init_5, property_page_provider_get_pages_5, PROPERTY_PAGE_PROVIDER_5, set_property_page_provider_5);
property_page_provider_iface!(property_page_provider_iface_init_6, property_page_provider_get_pages_6, PROPERTY_PAGE_PROVIDER_6, set_property_page_provider_6);
property_page_provider_iface!(property_page_provider_iface_init_7, property_page_provider_get_pages_7, PROPERTY_PAGE_PROVIDER_7, set_property_page_provider_7);
property_page_provider_iface!(property_page_provider_iface_init_8, property_page_provider_get_pages_8, PROPERTY_PAGE_PROVIDER_8, set_property_page_provider_8);
property_page_provider_iface!(property_page_provider_iface_init_9, property_page_provider_get_pages_9, PROPERTY_PAGE_PROVIDER_9, set_property_page_provider_9);

pub fn property_page_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        property_page_provider_iface_init_0,
        property_page_provider_iface_init_1,
        property_page_provider_iface_init_2,
        property_page_provider_iface_init_3,
        property_page_provider_iface_init_4,
        property_page_provider_iface_init_5,
        property_page_provider_iface_init_6,
        property_page_provider_iface_init_7,
        property_page_provider_iface_init_8,
        property_page_provider_iface_init_9,
    ]
}

pub fn rust_property_page_provider_setters() -> Vec<fn(Box<dyn PropertyPageProvider>)> {
    vec![
        set_property_page_provider_0,
        set_property_page_provider_1,
        set_property_page_provider_2,
        set_property_page_provider_3,
        set_property_page_provider_4,
        set_property_page_provider_5,
        set_property_page_provider_6,
        set_property_page_provider_7,
        set_property_page_provider_8,
        set_property_page_provider_9,
    ]
}

static NEXT_PROPERTY_PAGE_PROVIDER_IFACE_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn take_next_property_page_provider_iface_index() -> usize {
    NEXT_PROPERTY_PAGE_PROVIDER_IFACE_INDEX.fetch_add(1, Ordering::SeqCst)
}
