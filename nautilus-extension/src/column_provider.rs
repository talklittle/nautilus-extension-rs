use glib_ffi::{GList, gpointer};
use std::borrow::Cow;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Column {
    pub name: Cow<'static, str>,
    pub attribute: Cow<'static, str>,
    pub label: Cow<'static, str>,
    pub description: Cow<'static, str>,
}

impl Column {
    pub fn new<S: Into<Cow<'static, str>>>(name: S, attribute: S, label: S, description: S) -> Column {
        Column {
            name: name.into(),
            attribute: attribute.into(),
            label: label.into(),
            description: description.into(),
        }
    }
}

pub trait ColumnProvider : Send + Sync {
    fn get_columns(&self) -> Vec<Column>;
}

macro_rules! column_provider_iface {
    ($iface_init_fn:ident, $get_columns_fn:ident, $rust_provider:ident, $set_rust_provider:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            use nautilus_ffi::NautilusColumnProviderIface;

            let iface_struct = iface as *mut NautilusColumnProviderIface;
            (*iface_struct).get_columns = Some($get_columns_fn);
        }

        #[no_mangle]
        pub extern "C" fn $get_columns_fn(_provider: gpointer) -> *mut GList {
            use glib_ffi::g_list_append;
            use libc::c_void;
            use nautilus_ffi::nautilus_column_new;
            use std::ffi::CString;
            use std::ptr;

            let mut columns_g_list = ptr::null_mut();

            let columns = match *$rust_provider.lock().unwrap() {
                Some(ref p) => p.get_columns(),
                None => Vec::new(),
            };

            for column in columns {
                let name = CString::new(&column.name as &str).unwrap().into_raw();
                let attribute = CString::new(&column.attribute as &str).unwrap().into_raw();
                let label = CString::new(&column.label as &str).unwrap().into_raw();
                let description = CString::new(&column.description as &str).unwrap().into_raw();

                unsafe {
                    let column_c = nautilus_column_new(name, attribute, label, description);
                    columns_g_list = g_list_append(columns_g_list, column_c as *mut c_void);

                    // deallocate CStrings
                    let _ = CString::from_raw(name);
                    let _ = CString::from_raw(attribute);
                    let _ = CString::from_raw(label);
                    let _ = CString::from_raw(description);
                }
            }

            columns_g_list
        }

        pub fn $set_rust_provider(column_provider: Box<dyn ColumnProvider>) {
            *$rust_provider.lock().unwrap() = Some(column_provider);
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Box<dyn ColumnProvider>>> = Mutex::new(None);
        }
    }
}

// Let library consumer add up to 10 ColumnProviders, should be more than enough. Each has its own Vec of columns.
column_provider_iface!(column_provider_iface_init_0, column_provider_get_columns_0, COLUMN_PROVIDER_0, set_column_provider_0);
column_provider_iface!(column_provider_iface_init_1, column_provider_get_columns_1, COLUMN_PROVIDER_1, set_column_provider_1);
column_provider_iface!(column_provider_iface_init_2, column_provider_get_columns_2, COLUMN_PROVIDER_2, set_column_provider_2);
column_provider_iface!(column_provider_iface_init_3, column_provider_get_columns_3, COLUMN_PROVIDER_3, set_column_provider_3);
column_provider_iface!(column_provider_iface_init_4, column_provider_get_columns_4, COLUMN_PROVIDER_4, set_column_provider_4);
column_provider_iface!(column_provider_iface_init_5, column_provider_get_columns_5, COLUMN_PROVIDER_5, set_column_provider_5);
column_provider_iface!(column_provider_iface_init_6, column_provider_get_columns_6, COLUMN_PROVIDER_6, set_column_provider_6);
column_provider_iface!(column_provider_iface_init_7, column_provider_get_columns_7, COLUMN_PROVIDER_7, set_column_provider_7);
column_provider_iface!(column_provider_iface_init_8, column_provider_get_columns_8, COLUMN_PROVIDER_8, set_column_provider_8);
column_provider_iface!(column_provider_iface_init_9, column_provider_get_columns_9, COLUMN_PROVIDER_9, set_column_provider_9);

pub fn column_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        column_provider_iface_init_0,
        column_provider_iface_init_1,
        column_provider_iface_init_2,
        column_provider_iface_init_3,
        column_provider_iface_init_4,
        column_provider_iface_init_5,
        column_provider_iface_init_6,
        column_provider_iface_init_7,
        column_provider_iface_init_8,
        column_provider_iface_init_9,
    ]
}

pub fn rust_column_provider_setters() -> Vec<fn(Box<dyn ColumnProvider>)> {
    vec![
        set_column_provider_0,
        set_column_provider_1,
        set_column_provider_2,
        set_column_provider_3,
        set_column_provider_4,
        set_column_provider_5,
        set_column_provider_6,
        set_column_provider_7,
        set_column_provider_8,
        set_column_provider_9,
    ]
}

static NEXT_COLUMN_PROVIDER_IFACE_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn take_next_column_provider_iface_index() -> usize {
    NEXT_COLUMN_PROVIDER_IFACE_INDEX.fetch_add(1, Ordering::SeqCst)
}
