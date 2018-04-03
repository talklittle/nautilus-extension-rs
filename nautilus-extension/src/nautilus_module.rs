use column_provider::{ColumnProvider, take_next_column_provider_iface_index, column_provider_iface_externs, rust_column_provider_setters};
use glib_ffi::GType;
use gobject_ffi::{GInterfaceInfo, GObjectClass, GTypeFlags, GTypeInfo, GTypeModule, GTypeQuery, GTypeValueTable};
use gobject_ffi::{g_type_module_add_interface, g_type_module_register_type, g_type_query};
use gobject_ffi::G_TYPE_OBJECT;
use info_provider::{InfoProvider, take_next_info_provider_iface_index, info_provider_iface_externs, rust_info_provider_setters};
use libc::c_char;
use menu_provider::{MenuProvider, take_next_menu_provider_iface_index, menu_provider_iface_externs, rust_menu_provider_setters};
use nautilus_ffi::{nautilus_column_provider_get_type, nautilus_info_provider_get_type, nautilus_menu_provider_get_type, nautilus_property_page_provider_get_type};
use property_page_provider::{PropertyPageProvider, take_next_property_page_provider_iface_index, property_page_provider_iface_externs, rust_property_page_provider_setters};
use std::ffi::CString;
use std::mem;
use std::ptr;

#[repr(C)]
struct NautilusExtensionClass {
    _parent_slot: GObjectClass
}

const EMPTY_VALUE_TABLE: GTypeValueTable = GTypeValueTable {
    value_init: None,
    value_free: None,
    value_copy: None,
    value_peek_pointer: None,
    collect_format: 0 as *const c_char,
    collect_value: None,
    lcopy_format: 0 as *const c_char,
    lcopy_value: None
};

pub struct NautilusModule {
    module: *mut GTypeModule,
    name: String,
    column_provider_iface_infos: Vec<GInterfaceInfo>,
    info_provider_iface_infos: Vec<GInterfaceInfo>,
    menu_provider_iface_infos: Vec<GInterfaceInfo>,
    property_page_provider_iface_infos: Vec<GInterfaceInfo>,
}

impl NautilusModule {
    pub fn new(module: *mut GTypeModule, name: &str) -> NautilusModule {
        NautilusModule {
            module: module,
            name: name.to_string(),
            column_provider_iface_infos: Vec::new(),
            info_provider_iface_infos: Vec::new(),
            menu_provider_iface_infos: Vec::new(),
            property_page_provider_iface_infos: Vec::new(),
        }
    }

    pub fn add_column_provider<'a, T: ColumnProvider + 'static>(&'a mut self, column_provider: T) -> &'a mut NautilusModule {
        let index = take_next_column_provider_iface_index();
        let iface_init_fn = column_provider_iface_externs()[index];
        let ref rust_provider_setter = rust_column_provider_setters()[index];

        let column_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(column_provider));

        self.column_provider_iface_infos.push(column_provider_iface_info);

        self
    }

    pub fn add_info_provider<'a, T: InfoProvider + 'static>(&'a mut self, info_provider: T) -> &'a mut NautilusModule {
        let index = take_next_info_provider_iface_index();
        let iface_init_fn = info_provider_iface_externs()[index];
        let ref rust_provider_setter = rust_info_provider_setters()[index];

        let info_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(info_provider));

        self.info_provider_iface_infos.push(info_provider_iface_info);

        self
    }

    pub fn add_menu_provider<'a, T: MenuProvider + 'static>(&'a mut self, menu_provider: T) -> &'a mut NautilusModule {
        let index = take_next_menu_provider_iface_index();
        let iface_init_fn = menu_provider_iface_externs()[index];
        let ref rust_provider_setter = rust_menu_provider_setters()[index];

        let menu_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(menu_provider));

        self.menu_provider_iface_infos.push(menu_provider_iface_info);

        self
    }

    pub fn add_property_page_provider<'a, T: PropertyPageProvider + 'static>(&'a mut self, property_page_provider: T) -> &'a mut NautilusModule {
        let index = take_next_property_page_provider_iface_index();
        let iface_init_fn = property_page_provider_iface_externs()[index];
        let ref rust_provider_setter = rust_property_page_provider_setters()[index];

        let property_page_provider_iface_info = GInterfaceInfo {
            interface_init: Some(iface_init_fn),
            interface_finalize: None,
            interface_data: ptr::null_mut(),
        };

        rust_provider_setter(Box::new(property_page_provider));

        self.property_page_provider_iface_infos.push(property_page_provider_iface_info);

        self
    }

    pub fn register(&self) -> GType {
        let name = CString::new(self.name.as_str()).unwrap();

        let info = GTypeInfo {
            class_size: mem::size_of::<NautilusExtensionClass>() as u16,
            base_init: None,
            base_finalize: None,
            class_init: None,
            class_finalize: None,
            class_data: ptr::null(),
            instance_size: g_object_instance_size(),
            n_preallocs: 0,
            instance_init: None,
            value_table: &EMPTY_VALUE_TABLE
        };

        unsafe {
            let module_type = g_type_module_register_type(self.module, G_TYPE_OBJECT, name.as_ptr(), &info, GTypeFlags::empty());

            for column_provider_iface_info in &self.column_provider_iface_infos {
                g_type_module_add_interface(self.module, module_type, nautilus_column_provider_get_type(), column_provider_iface_info);
            }

            for info_provider_iface_info in &self.info_provider_iface_infos {
                g_type_module_add_interface(self.module, module_type, nautilus_info_provider_get_type(), info_provider_iface_info);
            }

            for menu_provider_iface_info in &self.menu_provider_iface_infos {
                g_type_module_add_interface(self.module, module_type, nautilus_menu_provider_get_type(), menu_provider_iface_info);
            }

            for property_page_provider_iface_info in &self.property_page_provider_iface_infos {
                g_type_module_add_interface(self.module, module_type, nautilus_property_page_provider_get_type(), property_page_provider_iface_info);
            }

            module_type
        }
    }
}

fn g_object_instance_size() -> u16 {
    let mut query: GTypeQuery = GTypeQuery {
        instance_size: 0,
        class_size: 0,
        type_name: 0 as *const c_char,
        type_: 0
    };
    unsafe {
        g_type_query(G_TYPE_OBJECT, &mut query);
    }
    return query.instance_size as u16;
}
