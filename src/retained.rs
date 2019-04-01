
use std::cmp;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Mutex;
use std::u32;

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

pub struct Surface {
    hwnd: HWND,
    vulkan_surface: SurfaceKHR,
    swapchain: SwapchainKHR,
}

impl Surface {
    pub unsafe fn from_image(image: Image) -> Self {
        unimplemented!()
    }

    pub unsafe fn from_hwnd(hwnd: HWND) -> Self {
        let globals = VULKAN_GLOBALS.lock().unwrap();

        // TODO: cache?
        let win32_surface_loader = Win32Surface::new(&globals.entry, &globals.instance);
        let create_info = Win32SurfaceCreateInfoKHR {
            s_type: StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
            hinstance: GetModuleHandleW(ptr::null()) as *const c_void,
            hwnd: hwnd as *const c_void,
            ..Default::default()
        };
        let vulkan_surface = win32_surface_loader.create_win32_surface(&create_info, None)
            .expect("failed to create window surface");
        if !globals.surface_loader.get_physical_device_surface_support(
            globals.device.physical,
            globals.device.queue_family_indices.present,
            vulkan_surface,
        ) {
            panic!("physical device doesn't support presentation onto surface");
        }

        let swapchain = Self::create_swapchain(
            &globals,
            vulkan_surface,
            || unimplemented!(),
            SwapchainKHR::null(),
        );
        let swapchain_images = globals.device.swapchain_loader.get_swapchain_images(swapchain)
            .expect("failed to get swapchain images");

        Self {
            hwnd,
            vulkan_surface,
            swapchain,
        }
    }

    // `get_surface_extent` is only used as a fallback when the surface doesn't report a size. I
    // son't think it's needed on Windows and Linux at least.
    // `old_swapchain` should be SwapchainKHR::null() if there is no existing swapchain.
    unsafe fn create_swapchain<F>(
        globals: &VulkanGlobals,
        surface: SurfaceKHR,
        get_surface_extent: F,
        old_swapchain: SwapchainKHR,
    ) -> SwapchainKHR
    where F: FnOnce() -> Extent2D,
    {
        let surface_loader = &globals.surface_loader;
        let caps = surface_loader
            .get_physical_device_surface_capabilities(globals.device.physical, surface)
            .expect("failed to get surface capabilities");
        let formats = surface_loader
            .get_physical_device_surface_formats(globals.device.physical, surface)
            .expect("failed to get surface formats");
        let present_modes = surface_loader
            .get_physical_device_surface_present_modes(globals.device.physical, surface)
            .expect("failed to get surface formats");

        let best_format = *formats.iter().find(|fmt| {
            // There are only two commonly supported formats, R8G8B8A8_SRGB and R8G8B8A8_UNORM.
            fmt.format == Format::R8G8B8A8_SRGB && fmt.color_space == ColorSpaceKHR::SRGB_NONLINEAR
        }).unwrap_or_else(|| &formats[0]);
        let image_count = if caps.max_image_count == 0 {
            caps.min_image_count + 1
        } else {
            cmp::min(caps.min_image_count + 1, caps.max_image_count)
        };
        let image_extent = match caps.current_extent {
            Extent2D { width: u32::MAX, height: u32::MAX } => get_surface_extent(),
            _ => caps.current_extent,
        };
        // Just require these because everything I've looked at supports them and they may be
        // useful.
        let image_usage_extra = if !caps.supported_usage_flags.contains(ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::TRANSFER_SRC) {
            panic!("swapchain images must support TRANSFER_DST and TRANSFER_SRC");
        };
        // TODO: avoid this memory allocations
        let queue_family_indices = globals.device.queue_family_indices.get_unique_indices();
        let image_sharing_mode = if queue_family_indices.len() == 1 {
            SharingMode::EXCLUSIVE
        } else {
            SharingMode::CONCURRENT
        };

        let create_info = SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(best_format.format)
            .image_color_space(best_format.color_space)
            .image_extent(image_extent)
            .image_array_layers(1)
            .image_usage(
                ImageUsageFlags::COLOR_ATTACHMENT |
                ImageUsageFlags::TRANSFER_DST |
                ImageUsageFlags::TRANSFER_SRC,
            )
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(caps.current_transform)
            .composite_alpha(caps.supported_composite_alpha) // usually OPAQUE or INHERIT
            .present_mode(PresentModeKHR::FIFO) // always supported
            .clipped(true)
            .old_swapchain(old_swapchain);

        globals.device.swapchain_loader.create_swapchain(&create_info, None)
            .expect("failed to create swapchain")
    }

    pub fn draw(scene: &Scene) {
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            let globals = VULKAN_GLOBALS.lock().unwrap();
            globals.surface_loader.destroy_surface(self.vulkan_surface, None);
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
