use glib_ffi::gpointer;
use gobject_ffi::{GClosure, g_closure_ref};
use nautilus_ffi::{NautilusFileInfo, NautilusInfoProvider, NautilusOperationHandle, NautilusOperationResult};
use nautilus_ffi::{nautilus_file_info_add_string_attribute, nautilus_file_info_get_uri, nautilus_file_info_get_uri_scheme};
use nautilus_ffi::nautilus_file_info_invalidate_extension_info;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};

pub trait InfoProvider : Send + Sync {
    fn should_update_file_info(&self, &FileInfo) -> bool;
    fn update_file_info(&self, &mut FileInfo);
}

#[derive(Clone)]
pub struct FileInfo {
    pub raw_file_info: *mut NautilusFileInfo,
    pub attributes: HashMap<String, String>,
}

unsafe impl Send for FileInfo {}

impl FileInfo {
    pub fn new(raw_file_info: *mut NautilusFileInfo) -> FileInfo {
        FileInfo {
            raw_file_info: raw_file_info,
            attributes: HashMap::new(),
        }
    }

    pub fn get_uri(&self) -> String {
        let ref raw_file_info = self.raw_file_info;
        unsafe {
            CStr::from_ptr(nautilus_file_info_get_uri(*raw_file_info)).to_string_lossy().into_owned()
        }
    }

    pub fn get_uri_scheme(&self) -> String {
        let ref raw_file_info = self.raw_file_info;
        unsafe {
            CStr::from_ptr(nautilus_file_info_get_uri_scheme(*raw_file_info)).to_string_lossy().into_owned()
        }
    }

    pub fn invalidate_extension_info(&self) {
        unsafe {
            nautilus_file_info_invalidate_extension_info(self.raw_file_info);
        }
    }

    pub fn add_attribute(&mut self, name: &str, value: &str) -> &mut FileInfo {
        self.attributes.insert(name.to_string(), value.to_string());
        self
    }
}

pub struct UpdateFileInfoOperationHandle {
    pub skip_response: bool
}

macro_rules! info_provider_iface {
    ($iface_init_fn:ident, $update_file_info_fn:ident, $update_file_info_bg_fn:ident, $cancel_update_fn:ident, $rust_provider:ident, $set_rust_provider:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $iface_init_fn(iface: gpointer, _: gpointer) {
            use nautilus_ffi::NautilusInfoProviderIface;

            let iface_struct = iface as *mut NautilusInfoProviderIface;
            (*iface_struct).update_file_info = Some($update_file_info_fn);
            (*iface_struct).cancel_update = Some($cancel_update_fn);
        }

        #[no_mangle]
        pub extern "C" fn $update_file_info_fn(provider: *mut NautilusInfoProvider,
                                               file: *mut NautilusFileInfo,
                                               update_complete: *mut GClosure,
                                               handle: *mut *mut NautilusOperationHandle) -> NautilusOperationResult {
            use std::mem;
            use std::sync::{Arc, Mutex};
            use std::sync::mpsc::channel;
            use std::thread;

            let mut file_info = FileInfo::new(file);

            let should_update_file_info = match *$rust_provider.lock().unwrap() {
                Some(ref p) => p.should_update_file_info(&mut file_info),
                None => false,
            };

            if !should_update_file_info {
                return NautilusOperationResult::NautilusOperationComplete;
            }

            let my_handle = Arc::new(Mutex::new(UpdateFileInfoOperationHandle { skip_response: false }));
            let my_handle_local = my_handle.clone();
            let my_handle_thread = my_handle.clone();

            let (tx, rx) = channel();
            thread::spawn(move || {
                let (file_info, provider, update_complete, handle_ref) = rx.recv().unwrap();
                $update_file_info_bg_fn(file_info, provider, update_complete, my_handle_thread, handle_ref);
            });

            unsafe {
                let closure_copy = g_closure_ref(update_complete);
                *handle = mem::transmute(Box::into_raw(Box::new(my_handle_local)));
                tx.send((Box::new(file_info), &mut *provider, &mut *closure_copy, &mut **handle)).unwrap();
            }

            return NautilusOperationResult::NautilusOperationInProgress;
        }

        #[no_mangle]
        pub extern "C" fn $cancel_update_fn(_provider: *mut NautilusInfoProvider, handle: *mut NautilusOperationHandle) {
            use std::sync::{Arc, Mutex};

            unsafe {
                let handle = handle as *mut Arc<Mutex<UpdateFileInfoOperationHandle>>;
                let mut my_handle = (*handle).lock().unwrap();
                my_handle.skip_response = true;
            }
        }

        fn $update_file_info_bg_fn(mut file_info: Box<FileInfo>,
                            provider: &mut NautilusInfoProvider,
                            update_complete: &mut GClosure,
                            my_handle: Arc<Mutex<UpdateFileInfoOperationHandle>>,
                            handle_ref: &mut NautilusOperationHandle) {
            use nautilus_ffi::nautilus_info_provider_update_complete_invoke;

            match *$rust_provider.lock().unwrap() {
                Some(ref p) => p.update_file_info(file_info.as_mut()),
                None => (),
            };

            if !my_handle.lock().unwrap().skip_response {
                let file_info = file_info.as_mut();
                let ref attributes = file_info.attributes;
                unsafe {
                    for (attr_name, attr_value) in attributes {
                        let attr_name_c = CString::new(attr_name.as_str()).unwrap().into_raw();
                        let attr_value_c = CString::new(attr_value.as_str()).unwrap().into_raw();

                        nautilus_file_info_add_string_attribute(file_info.raw_file_info, attr_name_c, attr_value_c);

                        // deallocate CStrings
                        CString::from_raw(attr_name_c);
                        CString::from_raw(attr_value_c);
                    }

                    nautilus_info_provider_update_complete_invoke(update_complete, provider, handle_ref, NautilusOperationResult::NautilusOperationComplete);
                }
            }
        }

        pub fn $set_rust_provider(info_provider: Box<InfoProvider>) {
            *$rust_provider.lock().unwrap() = Some(info_provider);
        }

        lazy_static! {
            static ref $rust_provider: Mutex<Option<Box<InfoProvider>>> = Mutex::new(None);
        }
    }
}

info_provider_iface!(info_provider_iface_init_0, info_provider_update_file_info_0, info_provider_update_file_info_bg_0, info_provider_cancel_update_0, INFO_PROVIDER_0, set_info_provider_0);
info_provider_iface!(info_provider_iface_init_1, info_provider_update_file_info_1, info_provider_update_file_info_bg_1, info_provider_cancel_update_1, INFO_PROVIDER_1, set_info_provider_1);
info_provider_iface!(info_provider_iface_init_2, info_provider_update_file_info_2, info_provider_update_file_info_bg_2, info_provider_cancel_update_2, INFO_PROVIDER_2, set_info_provider_2);
info_provider_iface!(info_provider_iface_init_3, info_provider_update_file_info_3, info_provider_update_file_info_bg_3, info_provider_cancel_update_3, INFO_PROVIDER_3, set_info_provider_3);
info_provider_iface!(info_provider_iface_init_4, info_provider_update_file_info_4, info_provider_update_file_info_bg_4, info_provider_cancel_update_4, INFO_PROVIDER_4, set_info_provider_4);
info_provider_iface!(info_provider_iface_init_5, info_provider_update_file_info_5, info_provider_update_file_info_bg_5, info_provider_cancel_update_5, INFO_PROVIDER_5, set_info_provider_5);
info_provider_iface!(info_provider_iface_init_6, info_provider_update_file_info_6, info_provider_update_file_info_bg_6, info_provider_cancel_update_6, INFO_PROVIDER_6, set_info_provider_6);
info_provider_iface!(info_provider_iface_init_7, info_provider_update_file_info_7, info_provider_update_file_info_bg_7, info_provider_cancel_update_7, INFO_PROVIDER_7, set_info_provider_7);
info_provider_iface!(info_provider_iface_init_8, info_provider_update_file_info_8, info_provider_update_file_info_bg_8, info_provider_cancel_update_8, INFO_PROVIDER_8, set_info_provider_8);
info_provider_iface!(info_provider_iface_init_9, info_provider_update_file_info_9, info_provider_update_file_info_bg_9, info_provider_cancel_update_9, INFO_PROVIDER_9, set_info_provider_9);

pub fn info_provider_iface_externs() -> Vec<unsafe extern "C" fn(gpointer, gpointer)> {
    vec![
        info_provider_iface_init_0,
        info_provider_iface_init_1,
        info_provider_iface_init_2,
        info_provider_iface_init_3,
        info_provider_iface_init_4,
        info_provider_iface_init_5,
        info_provider_iface_init_6,
        info_provider_iface_init_7,
        info_provider_iface_init_8,
        info_provider_iface_init_9,
    ]
}

pub fn rust_info_provider_setters() -> Vec<fn(Box<InfoProvider>)> {
    vec![
        set_info_provider_0,
        set_info_provider_1,
        set_info_provider_2,
        set_info_provider_3,
        set_info_provider_4,
        set_info_provider_5,
        set_info_provider_6,
        set_info_provider_7,
        set_info_provider_8,
        set_info_provider_9,
    ]
}

static mut NEXT_INFO_PROVIDER_IFACE_INDEX: usize = 0;

pub fn take_next_info_provider_iface_index() -> usize {
    unsafe {
        let result = NEXT_INFO_PROVIDER_IFACE_INDEX;
        NEXT_INFO_PROVIDER_IFACE_INDEX += 1;
        result
    }
}
