
use std::os::raw::c_void;
use std::ptr;
use std::sync::Mutex;

use ash::{Entry, Instance};
use ash::extensions::khr::Win32Surface;
use ash::prelude::VkResult;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk::*;
use winapi::shared::windef::{*, HWND};
use winapi::um::libloaderapi::GetModuleHandleW;

use super::{Color, Point2, Rect, Size2};
use super::vk_util::{create_instance, get_device_extensions_list, VulkanGlobals};

/// An image stored on the CPU.
pub struct Image {
}

impl Image {
    fn size(&self) -> Size2<f64> {
        unimplemented!()
    }
}

/// An image stored on the GPU. On systems with unified memory between the CPU and GPU, converting
/// from Image to a GpuImage is a noop.
pub struct GpuImage {
}

pub struct TextLayout {
}

// After looking at Qt, WPF, and UIKit, the naming stretch, fit, and fill are mainly inspired by
// Windows 10 background settings and UIKit. I thought "stretch" would be good, and UIKit uses "fit"
// and "fill". Windows 10 background settings calling them simply that made me go with it. macOS
// background settings uses "Fill Screen", "Fit to Screen", and "Stretch to Fill Screen", so the
// same three verbs.

/// Determines how an image is scaled when its aspect ratio doesn't match the aspect ratio of a
/// destination area.
#[derive(Copy, Clone, Debug)]
pub enum ScalingMode {
    /// The image is scaled to fill the destination, changing the aspect ratio if necessary.
    Stretch,
    /// The image is scaled to fill the destination on one axis, preserving the aspect ratio. Any
    /// remaining area of the destination is transparent.
    Fit,
    /// The image is scaled to fill the destination, preserving the aspect ratio by cropping the
    /// image if necessary.
    Fill,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
enum Brush {
    Solid(Color<f32>),
    LinearGradient(LinearGradient),
    //RadialGradient(),
    //MeshGradient(),
    //Bitmap/Image(),
}

#[derive(Clone, Debug)]
#[non_exhaustive]
struct LinearGradient {
    pub start_point: Point2<f64>,
    pub end_point: Point2<f64>,
    // If true, then the gradient is blended in the sRGB color space instead of using gamma-correct
    // linear interpolation. Using non-linear interpolation is wrong, but may be useful for
    // compatibility.
    // https://docs.microsoft.com/en-us/windows/desktop/api/d2d1/ne-d2d1-d2d1_gamma
    pub incorrect_gamma_blending: bool, // TODO: move to a GradientStopList like Direct2D?
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    pub fn new(start_point: Point2<f64>, end_point: Point2<f64>) -> Self {
        Self {
            start_point,
            end_point,
            incorrect_gamma_blending: false,
            stops: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color<f32>,
}

pub enum DrawCommand {
    DrawRect,
    FillRect,
    DrawPath,
    FillPath,
    Image {
        image: Image,
        dest_rect: Rect<f64>,
        src_rect: Rect<f64>,
        opacity: f32,
    },
    Text(Box<str>, Rect<f64>),
}

impl DrawCommand {
    pub fn rect() -> Self {
        DrawCommand::DrawRect
    }

    pub fn image(
        image: Image,
        dest_rect: Rect<f64>,
    ) -> Self {
        let image_size = image.size();
        DrawCommand::Image {
            image,
            dest_rect,
            src_rect: Point2::new(0.0, 0.0) + image_size,
            opacity: 1.0,
            //scaling_mode: ScalingMode::Fit,
        }
    }

    pub fn image_with_options(
        image: Image,
        dest_rect: Rect<f64>,
        src_rect: Rect<f64>,
        opacity: f32,
        scaling_mode: ScalingMode,
    ) -> Self {
        DrawCommand::Image { image, dest_rect, src_rect, opacity }
    }
}

lazy_static! {
    static ref VULKAN_GLOBALS: Mutex<VulkanGlobals> = Mutex::new(create_instance());
}

#[derive(Copy, Clone, Debug)]
struct QueueFamilyIndexes {
    graphics: u32,
    present: u32,
    transfer: u32,
}

impl QueueFamilyIndexes {
    fn get_unique_indexes(&self) -> Vec<u32> {
        let mut indexes = Vec::with_capacity(3);
        indexes.push(self.graphics);
        if !indexes.contains(&self.present) {
            indexes.push(self.present);
        }
        if !indexes.contains(&self.transfer) {
            indexes.push(self.transfer);
        }
        indexes
    }
}

pub struct Surface {
    hwnd: HWND,
    vulkan_surface: SurfaceKHR,
    device: ash::Device,
}

impl Surface {
    pub unsafe fn from_image(image: Image) -> Self {
        unimplemented!()
    }

    pub unsafe fn from_hwnd(hwnd: HWND) -> Self {
        let globals = VULKAN_GLOBALS.lock().unwrap();
        let win32_surface_loader = Win32Surface::new(&globals.entry, &globals.instance);
        let create_info = Win32SurfaceCreateInfoKHR {
            s_type: StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            hinstance: GetModuleHandleW(ptr::null()) as *const c_void,
            hwnd: hwnd as *const c_void,
            ..Default::default()
        };
        let vulkan_surface = win32_surface_loader.create_win32_surface(&create_info, None)
            .expect("failed to create window surface");
        let (phy_device, indexes) = Self::get_preferred_physical_device(&globals, vulkan_surface)
            .expect("no acceptable physical device found");

        const QUEUE_PRIORITIES: &[f32] = &[1.0]; // only create one queue
        let queue_create_infos: Vec<_> = indexes.get_unique_indexes().iter().map(|i|
            DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(QUEUE_PRIORITIES) // sets queue count too
                .build()
        ).collect();
        let extensions = get_device_extensions_list(&globals.instance, phy_device);
        let enabled_features = PhysicalDeviceFeatures::builder().build();
        let device_create_info = DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&enabled_features)
            .build();
        let device = globals.instance.create_device(phy_device, &device_create_info, None)
            .expect("failed to create logical device");

        Self {
            hwnd,
            vulkan_surface,
            device,
        }
    }

    unsafe fn get_preferred_physical_device(
        globals: &VulkanGlobals,
        surface: SurfaceKHR,
    ) -> Option<(PhysicalDevice, QueueFamilyIndexes)> {
        let surface_loader = ash::extensions::khr::Surface::new(&globals.entry, &globals.instance);
        let phy_devices = globals.instance.enumerate_physical_devices()
            .expect("failed to get physical devices");
        let filtered_devices: Vec<_> = phy_devices.iter().filter_map(|dev| {
            // vulkan-tutorial.com checks if queueCount > 0, but the Vulkan spec says "Each queue
            // family must support at least one queue." so I don't think that is necessary.
            let queue_family_props = globals.instance
                .get_physical_device_queue_family_properties(*dev);
            // Any impl that supports graphics must support compute too (but we aren't checking
            // for it yet). And graphics is a superset of transfer so shouldn't be checked for.
            let graphics_index = queue_family_props
                .iter()
                .position(|props| props.queue_flags.contains(QueueFlags::GRAPHICS));
            // Find a transfer queue that can use the GPU's copy engine.
            let transfer_index = queue_family_props
                .iter()
                .position(|props| {
                    props.queue_flags.contains(QueueFlags::TRANSFER) &&
                    !props.queue_flags.contains(QueueFlags::GRAPHICS)
                });
            // Don't require a separate transfer queue.
            let transfer_index = transfer_index.or(graphics_index);
            // Find any queue that supports presenting to the surface.
            let present_index = (0..queue_family_props.len() as u32)
                .find(|i| surface_loader.get_physical_device_surface_support(*dev, *i, surface));
            if let (Some(graphics_index), Some(present_index), Some(transfer_index)) =
                (graphics_index, present_index, transfer_index)
            {
                Some((*dev, QueueFamilyIndexes {
                    graphics: graphics_index as u32,
                    present: present_index as u32,
                    transfer: transfer_index as u32,
                }))
            } else {
                None
            }
        }).collect();
        for (dev, queue_family_indexes) in filtered_devices.iter() {
            let props = globals.instance.get_physical_device_properties(*dev);
            if props.device_type == PhysicalDeviceType::DISCRETE_GPU {
                return Some((*dev, *queue_family_indexes));
            }
        }
        if !filtered_devices.is_empty() {
            return Some(filtered_devices[0]);
        }
        None
    }

    pub fn draw(scene: &Scene) {
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            let globals = VULKAN_GLOBALS.lock().unwrap();
            let surface_loader =
                ash::extensions::khr::Surface::new(&globals.entry, &globals.instance);
            surface_loader.destroy_surface(self.vulkan_surface, None);
            self.device.destroy_device(None);
        }
    }
}

pub struct Scene {
    commands: Vec<DrawCommand>,
}

impl Scene {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }
    pub fn add_command(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
