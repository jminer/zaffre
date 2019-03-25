
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use ash::{Entry, Instance};
use ash::vk::*;
use ash::extensions::khr::{Surface, Swapchain, Win32Surface};
use ash::extensions::ext::DebugUtils;
use ash::version::{EntryV1_0, InstanceV1_0};

use smallvec::SmallVec;

// TODO: I should probably put this or a type that does the same thing into a crate, since I've
// copied it into four projects now (clear-coat, nightshade?, radiance, zaffre).
pub fn str_to_c_vec<'a: 'b, 'b, A: ::smallvec::Array<Item=u8>>(s: &'a str, buf: &'b mut SmallVec<A>) -> *const c_char {
    // `CString` in the std library doesn't check if the &str already ends in a null terminator
    // It allocates and pushes a 0 unconditionally. However, I can add the null to string literals
    // and avoid many allocations.
    if s.as_bytes().last() == Some(&0) && !s.as_bytes()[..s.len() - 1].contains(&b'\0') {
        s.as_bytes().as_ptr() as *const c_char
    } else {
        buf.grow(s.len() + 1);
        buf.extend(s.as_bytes().iter().map(|c| if *c == b'\0' { b'?' } else { *c }));
        buf.push(0);
        (&buf[..]).as_ptr() as *const c_char
    }
}

const VULKAN_VERSION: u32 = vk_make_version!(1, 0, 104);

const STANDARD_VALIDATION_LAYER_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_LUNARG_standard_validation\0") };

pub struct VulkanGlobals {
    pub entry: Entry,
    pub instance: Instance,
}

pub fn create_instance() -> VulkanGlobals {
    unsafe {
        // TODO: when Entry::new() fails, report that Vulkan may not be installed.
        let entry = Entry::new().unwrap();
        let app_info = ApplicationInfo {
            s_type: StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: ptr::null(),
            application_version: vk_make_version!(0, 0, 0),
            p_engine_name: "Zaffre\0".as_ptr() as *const c_char,
            engine_version: vk_make_version!(
                env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap(),
                env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap(),
                env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap()),
            api_version: VULKAN_VERSION,
        };
        let layers: Vec<*const c_char> = vec![];
        let extensions: Vec<*const c_char> = get_instance_extensions_list(&entry);
        let create_info = InstanceCreateInfo {
            s_type: StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: InstanceCreateFlags::from_raw(0),
            p_application_info: &app_info,
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: layers.as_ptr() as *const *const c_char,
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr() as *const *const c_char,
        };
        let instance = entry.create_instance(&create_info, None)
            .expect("failed to create instance");
        VulkanGlobals { entry, instance }
    }
}

#[cfg(windows)]
fn native_surface_ext() -> &'static CStr {
    Win32Surface::name()
}

fn get_instance_extensions_list(entry: &Entry) -> Vec<*const c_char> {
    let mut extensions = vec![
        Surface::name(), native_surface_ext()
    ];
    println!("Vulkan instance extensions:");
    let ext_props = entry.enumerate_instance_extension_properties()
        .expect("failed to get extension properties");
    for ext_prop in ext_props {
        let ext_name = unsafe { CStr::from_ptr(ext_prop.extension_name.as_ptr()) };
        println!("  {}", ext_name.to_string_lossy());
        if cfg!(debug_assertions) {
            // DebugUtils is the replacement for previous debug extensions.
            if ext_name == DebugUtils::name() {
                extensions.push(DebugUtils::name());
            }
        }
    }
    extensions.iter().map(|ext| ext.as_ptr()).collect()
}

pub unsafe fn get_device_extensions_list(
    instance: &Instance,
    phy_device: PhysicalDevice,
) -> Vec<*const c_char> {
    let mut extensions = vec![
        Swapchain::name(),
    ];
    println!("Vulkan device extensions:");
    let ext_props = instance.enumerate_device_extension_properties(phy_device)
        .expect("failed to get extension properties");
    for ext_prop in ext_props {
        let ext_name = unsafe { CStr::from_ptr(ext_prop.extension_name.as_ptr()) };
        println!("  {}", ext_name.to_string_lossy());
    }
    extensions.iter().map(|ext| ext.as_ptr()).collect()
}