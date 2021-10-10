
use std::cmp;
use std::os::raw::c_void;
use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::AtomicBool;
use std::ops::Deref;
use std::slice;
use std::u32;
use std::u64;

use ash::extensions::khr::Win32Surface;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk::*;
use once_cell::sync::Lazy;
use winapi::shared::windef::{*, HWND};
use winapi::um::libloaderapi::GetModuleHandleW;

use super::{Color, Point2, Rect, Size2};
use crate::vk_util::{create_instance, get_pipeline, PipelineArgs, VulkanGlobals};
use crate::vk_allocator::DeviceMemoryRef;

pub(crate) struct ImageCopyOp {
    cmd_buffer: CommandBuffer,
    fence: Fence,
}

pub(crate) struct ImageStatus {
    // Unused with buffers.
    layout: ImageLayout,

    current_op: Option<Arc<ImageCopyOp>>,
}

impl ImageStatus {
    pub(crate) fn finish_operation(&mut self) {
        let op = match self.current_op.as_ref() {
            Some(op) => op,
            None => return,
        };

        let globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();
        unsafe {
            globals.device.logical.wait_for_fences(&[op.fence], true, u64::MAX);
            // TODO: free cmd_buffer and fence
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
/// BCx formats are supported on desktops, and ETC and EAC formats on mobile.
pub enum ImageFormat {
    R8G8B8A8_Srgb,
    B8G8R8A8_Srgb,
    R16G16B16A16_Sfloat,

    BC6H_SfloatBlock,
    BC7_SrgbBlock,

    ETC2_R8G8B8_SrgbBlock,
    ETC2_R8G8B8A1_SrgbBlock,
    ETC2_R8G8B8A8_SrgbBlock,
}

impl ImageFormat {
    pub fn is_valid_size(self, size: Size2<u16>) -> bool {
        match self {
            ImageFormat::R8G8B8A8_Srgb |
            ImageFormat::B8G8R8A8_Srgb |
            ImageFormat::R16G16B16A16_Sfloat => true,

            ImageFormat::BC6H_SfloatBlock |
            ImageFormat::BC7_SrgbBlock |
            ImageFormat::ETC2_R8G8B8_SrgbBlock |
            ImageFormat::ETC2_R8G8B8A1_SrgbBlock |
            ImageFormat::ETC2_R8G8B8A8_SrgbBlock => size.width % 4 == 0 && size.height % 4 == 0,
            // ASTC has 4x4, 5x4, 8x6, 10x10, ..., 12x12 pixel blocks
        }
    }

    pub fn get_byte_size(self, size: Size2<u16>) -> usize {
        debug_assert!(self.is_valid_size(size));
        let (width, height) = (size.width as usize, size.height as usize);
        match self {
            ImageFormat::R8G8B8A8_Srgb |
            ImageFormat::B8G8R8A8_Srgb => width * height * 4,
            ImageFormat::R16G16B16A16_Sfloat => width * height * 8,

            ImageFormat::BC6H_SfloatBlock |
            ImageFormat::BC7_SrgbBlock => (width / 4) * (height / 4) * 16,

            ImageFormat::ETC2_R8G8B8_SrgbBlock |
            ImageFormat::ETC2_R8G8B8A1_SrgbBlock => (width / 4) * (height / 4) * 8,

            ImageFormat::ETC2_R8G8B8A8_SrgbBlock => (width / 4) * (height / 4) * 16,
        }
    }
}

/// An image stored in main memory (accessible by the CPU).
///
/// Warning: Dropping an `ImageBuf` will wait for any operation in progress to finish, blocking the
/// current thread until the GPU finishes.
pub struct ImageBuf {
    // Staging buffers are recommended over staging images (staging images may not support
    // compressed formats).
    // https://www.reddit.com/r/vulkan/comments/71k4gy/why_is_vk_image_tiling_linear_so_limited/dnchgcp/
    buffer: Buffer,
    format: ImageFormat,
    size: Size2<u16>,
    memory_ref: DeviceMemoryRef,
    data: *mut c_void,
    status: Mutex<ImageStatus>,
}

impl ImageBuf {
    pub fn new(size: Size2<u16>, format: ImageFormat) -> Self {
        unsafe {
            let globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();
            let device = &*globals.device.logical;

            assert!(format.is_valid_size(size));
            let byte_size = format.get_byte_size(size) as u64;
            let buffer = device.create_buffer(&BufferCreateInfo::builder()
                .size(byte_size)
                .usage(BufferUsageFlags::TRANSFER_SRC | BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(SharingMode::EXCLUSIVE), None).expect("failed to create buffer");
            let memory_req = device.get_buffer_memory_requirements(buffer);
            let memory_ref = globals.allocator.allocate(memory_req,
                MemoryPropertyFlags::HOST_VISIBLE);
            let data = device.map_memory(memory_ref.memory, memory_ref.offset, byte_size,
                MemoryMapFlags::empty())
                .expect("failed to map buffer memory");

            Self {
                buffer,
                format,
                size,
                memory_ref,
                data,
                status: Mutex::new(ImageStatus {
                    layout: ImageLayout::UNDEFINED,
                    current_op: None,
                }),
            }
        }
    }

    fn format(&self) -> ImageFormat {
        self.format
    }

    fn size(&self) -> Size2<u16> {
        self.size
    }

    // Returns a slice of the image data. If a GPU operation is currently accessing the image, this
    // method will block until the operation finishes.
    fn data(&self) -> &[u8] {
        let mut status = self.status.lock().unwrap();
        status.finish_operation();

        unsafe {
            slice::from_raw_parts(self.data as *const _, self.format.get_byte_size(self.size()))
        }
    }

    // Returns a mutable slice of the image data. If a GPU operation is currently accessing the
    // image, this method will block until the operation finishes.
    fn data_mut(&self) -> &mut [u8] {
        let mut status = self.status.lock().unwrap();
        status.finish_operation();

        unsafe {
            slice::from_raw_parts_mut(self.data as *mut _, self.format.get_byte_size(self.size()))
        }
    }

    // TODO: maybe this method should take a Rect to copy on both the source and destination
    pub fn copy_to(self: Arc<ImageBuf>, dest: &GpuImageBuf) {
        assert!(dest.size() == self.size());
        assert!(dest.format() == self.format());
        // TODO: have to call finish_operation for both source and dest images first

        // TODO: make sure to check minImageTransferGranularity is (1, 1, 1) somewhere before using
        // transfer queue
        // TODO: when to destroy command buffer and ImageBuf?
        // have a global cleanup list, and in Surface::draw, go through the global cleanup?
        unsafe {
            let globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();
            let device = &*globals.device.logical;
            let status = self.status.lock().unwrap();

            let cmd_buffer = device.allocate_command_buffers(
                &CommandBufferAllocateInfo::builder()
                    .command_pool(globals.graphics_command_pool)
                    .level(CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1))
                .expect("failed to allocate command buffer")[0];

            let regions = &[BufferImageCopy::builder()
                .buffer_offset(0)
                .image_subresource(
                    ImageSubresourceLayers::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .layer_count(1)
                        .build())
                .image_offset(Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(Extent3D {
                    width: self.size.width as u32,
                    height: self.size.height as u32,
                    depth: 1,
                })
                .build()];
            // TODO: have to make sure it's the correct layout or use a memory barrier to change
            device.cmd_copy_buffer_to_image(cmd_buffer,
                self.buffer, dest.vk_image, ImageLayout::TRANSFER_DST_OPTIMAL, regions);

            let transfer_queue = device.get_device_queue(
                globals.device.queue_family_indices.transfer, 0);
            let cmd_buffers = &[cmd_buffer];
            let fence = todo!();
            device.queue_submit(transfer_queue,
                &[SubmitInfo::builder()
                .command_buffers(cmd_buffers)
                // TODO: set signal_semaphores so the GPU can wait on that before drawing?
                .build()], fence)
                .expect("failed to submit copy to transfer queue");
        }
    }

    pub fn to_gpu_image(self: Arc<ImageBuf>) -> GpuImageBuf {
        let gpu_image = GpuImageBuf::new(self.size, self.format);
        self.copy_to(&gpu_image);
        gpu_image
    }
}

impl Drop for ImageBuf {
    fn drop(&mut self) {
        {
            let mut status = self.status.lock().unwrap();
            status.finish_operation();
        }

        unsafe {
            let globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();
            let device = &*globals.device.logical;
            // TODO: wait on any operation to finish

            device.unmap_memory(self.memory_ref.memory);
            globals.allocator.free(self.memory_ref);
            device.destroy_buffer(self.buffer, None);
        }
    }
}

// TODO: It may be a decent idea to separate long term images into one array of descriptor sets, and
// short term images into another array of descriptor sets.
enum GpuImageLifetime {
    // Long term images are ones such as icons in an app. It could potentially need to draw them the
    // entire lifetime of the app.
    LongTerm,
    // Short term images are ones such as previews of images in a file chooser, images shown in a
    // web page in a web browser, or images
    ShortTerm,
}

/// An image stored in graphics memory (accessible by the GPU). On systems with unified memory
/// between the CPU and GPU, converting from `ImageBuf` to a `GpuImageBuf` still copies the image
/// because the pixels in an `ImageBuf` are defined to be row-major order in memory, but pixels in a
/// `GpuImageBuf` are laid out in a driver-defined order for efficient access by the GPU.
pub struct GpuImageBuf {
    vk_image: ash::vk::Image,
    vk_image_view: ash::vk::ImageView,
    format: ImageFormat,
    size: Size2<u16>,
    memory_ref: DeviceMemoryRef,
    descriptior_set: DescriptorSet,
    // The index of the descriptor in the descriptor set.
    descriptior_set_index: u32,
}

impl GpuImageBuf {

    pub fn new(size: Size2<u16>, format: ImageFormat) -> Self {
        todo!()
    }

    fn size(&self) -> Size2<u16> {
        unimplemented!()
    }

    fn format(&self) -> ImageFormat {
        self.format
    }
}

pub struct GpuImage {
    image_buf: Arc<GpuImageBuf>,
    rect: Rect<u16>,
}

impl GpuImage {
    fn size(&self) -> Size2<u16> {
        self.image_buf.size()
    }
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
pub enum Brush {
    Solid(Color<f32>),
    LinearGradient(LinearGradient),
    //RadialGradient(),
    //MeshGradient(),
    //Bitmap/Image(), // only support whole images (arbitrarily scaled)
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LinearGradient {
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

// Vulkan (and other APIs) has no way to draw part of an image as a whole image. When you use a
// sampler on a whole image, you can define what you get when you read outside the bounds of the
// image, like solid black or transparent or the color of the border pixel. Bilinear filtering also
// reads outside the bounds along the edge. This limitation makes image atlases unusable when drawn
// scaled. A workaround I've read is to leave some transparent pixels between images in the atlas,
// but the number of pixels necessary would depend on how much you wanted to scale it, which is hard
// to know.
//
// For this reason, I'm only providing two drawing commands: `DrawCommand::Image`, that can draw
// part of an image but can't draw it scaled, and `DrawCommand::ScaledImage`, that can draw an image
// scaled but can't draw part of an image. And an image brush will only be able to draw a whole
// image, not part of one.

pub enum DrawCommand {
    DrawRect,
    FillRect,
    DrawPath,
    FillPath,
    Image {
        image: GpuImage,
        dest_point: Point2<f64>,
        src_rect: Rect<u16>,
        opacity: f32,
    },
    ScaledImage {
        image: Arc<GpuImageBuf>,
        dest_rect: Rect<f64>,
        opacity: f32,
        scaling_mode: ScalingMode,
    },
    Text(Box<str>, Rect<f64>),
}

impl DrawCommand {
    pub fn rect() -> Self {
        DrawCommand::DrawRect
    }

    pub fn image(
        image: GpuImage,
        dest_point: Point2<f64>,
    ) -> Self {
        let image_size = image.size();
        DrawCommand::Image {
            image,
            dest_point,
            src_rect: Point2::new(0, 0) + image_size,
            opacity: 1.0,
            //scaling_mode: ScalingMode::Fit,
        }
    }

    pub fn image_with_options(
        image: GpuImage,
        dest_point: Point2<f64>,
        src_rect: Rect<u16>,
        opacity: f32,
    ) -> Self {
        DrawCommand::Image { image, dest_point, src_rect, opacity }
    }

    pub fn scaled_image(
        image: Arc<GpuImageBuf>,
        dest_rect: Rect<f64>,
    ) -> Self {
        let image_size = image.size();
        DrawCommand::ScaledImage {
            image,
            dest_rect,
            opacity: 1.0,
            scaling_mode: ScalingMode::Fit,
        }
    }

    pub fn scaled_image_with_options(
        image: Arc<GpuImageBuf>,
        dest_rect: Rect<f64>,
        opacity: f32,
        scaling_mode: ScalingMode,
    ) -> Self {
        DrawCommand::ScaledImage { image, dest_rect, opacity, scaling_mode }
    }
}

pub(crate) static VULKAN_GLOBALS: Lazy<Mutex<VulkanGlobals>> =
    Lazy::new(|| Mutex::new(create_instance()));

pub struct SwapchainSurface {
    hwnd: HWND,
    vulkan_surface: SurfaceKHR,
    swapchain: SwapchainKHR,
    image_format: SurfaceFormatKHR,
    swapchain_images: Vec<Image>,
}

pub enum Surface {
    Image(Image, Format),
    Swapchain(SwapchainSurface),
}

impl Surface {
    pub unsafe fn from_image(image: Image) -> Self {
        unimplemented!()
    }

    pub unsafe fn from_hwnd(hwnd: HWND) -> Self {
        let globals = VULKAN_GLOBALS.lock().unwrap();

        // TODO: cache?
        let win32_surface_loader = Win32Surface::new(&globals.entry, &*globals.instance);
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

        drop(globals);
        let mut this = Self::Swapchain(SwapchainSurface {
            hwnd,
            vulkan_surface,
            swapchain: SwapchainKHR::null(),
            image_format: SurfaceFormatKHR::default(),
            swapchain_images: vec![],
        });
        this.recreate_swapchain();
        this
    }

    pub fn image_format(&self) -> Format {
        match self {
            Surface::Image(_, f) => *f,
            Surface::Swapchain(SwapchainSurface { image_format, .. }) => image_format.format,
        }
    }

    pub fn recreate_swapchain(&mut self) {
        unsafe {
            let mut swapchain_surface = match self {
                Surface::Swapchain(s) => s,
                _ => panic!(),
            };
            let globals = VULKAN_GLOBALS.lock().unwrap();
            globals.device.logical.device_wait_idle().expect("device_wait_idle() failed");
            let (swapchain, image_format) = Self::create_swapchain(
                &globals,
                swapchain_surface.vulkan_surface,
                || unimplemented!(),
                swapchain_surface.swapchain,
            );
            swapchain_surface.swapchain = swapchain;
            swapchain_surface.image_format = image_format;
            swapchain_surface.swapchain_images = globals.device.swapchain_loader
                .get_swapchain_images(swapchain_surface.swapchain)
                .expect("failed to get swapchain images");
        }
    }

    // `get_surface_extent` is only used as a fallback when the surface doesn't report a size. I
    // don't think it's needed on Windows and Linux at least.
    // `old_swapchain` should be SwapchainKHR::null() if there is no existing swapchain.
    unsafe fn create_swapchain<F>(
        globals: &VulkanGlobals,
        surface: SurfaceKHR,
        get_surface_extent: F,
        old_swapchain: SwapchainKHR,
    ) -> (SwapchainKHR, SurfaceFormatKHR)
    where F: FnOnce() -> Extent2D,
    {
        let surface_loader = &globals.surface_loader;
        let caps = surface_loader
            .get_physical_device_surface_capabilities(globals.device.physical, surface)
            .expect("failed to get surface capabilities");
        let formats = surface_loader
            .get_physical_device_surface_formats(globals.device.physical, surface)
            .expect("failed to get surface formats");
        // Just using FIFO for now, which is always supported.
        //let present_modes = surface_loader
        //    .get_physical_device_surface_present_modes(globals.device.physical, surface)
        //    .expect("failed to get surface formats");

        let best_format = *formats.iter().find(|fmt| {
            // There are few commonly supported formats:
            // B8G8R8A8_SRGB and B8G8R8A8_UNORM on desktop and
            // R8G8B8A8_SRGB and R8G8B8A8_UNORM on mobile.
            (fmt.format == Format::R8G8B8A8_SRGB || fmt.format == Format::B8G8R8A8_SRGB) &&
                fmt.color_space == ColorSpaceKHR::SRGB_NONLINEAR
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
        let image_usage_extra = ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::TRANSFER_SRC;
        if !caps.supported_usage_flags.contains(image_usage_extra) {
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
                ImageUsageFlags::COLOR_ATTACHMENT | image_usage_extra,
            )
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(caps.current_transform)
            .composite_alpha(caps.supported_composite_alpha) // usually OPAQUE or INHERIT
            .present_mode(PresentModeKHR::FIFO) // always supported
            .clipped(true)
            .old_swapchain(old_swapchain);

        (globals.device.swapchain_loader.create_swapchain(&create_info, None)
            .expect("failed to create swapchain"), best_format)
    }

    pub fn draw(&mut self, scene: &Scene) {
        unsafe {
            let globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();
            let device = &*globals.device.logical;

            let pipeline = 5;
            let (pipeline, pipeline_layout) = get_pipeline(&*device, PipelineArgs {
                color_attachment_format: self.image_format(),
                color_attachment_final_layout: if let Self::Swapchain(_) = self {
                    ImageLayout::PRESENT_SRC_KHR
                } else {
                    todo!()
                },
                polygon_mode: PolygonMode::FILL,
            });
            for cmd in scene.commands.iter() {
                match cmd {
                    DrawCommand::DrawRect => (),
                    DrawCommand::FillRect => (),
                    DrawCommand::DrawPath => (),
                    DrawCommand::FillPath => (),
                    DrawCommand::Image { image, dest_point, src_rect, opacity } => {

                    },
                    DrawCommand::ScaledImage { image, dest_rect, opacity, scaling_mode } => {

                    },
                    DrawCommand::Text(_, _) => (),
                }
            }

            let cmd_buffer = device.allocate_command_buffers(
                &CommandBufferAllocateInfo::builder()
                    .command_pool(globals.graphics_command_pool)
                    .level(CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1))
                .expect("failed to allocate command buffer")[0];


            let (dst_image, dst_image_index) = match self {
                Surface::Image(image, _) => (*image, u32::MAX),
                Surface::Swapchain(swapchain_surface) => {
                    // TODO: need to use a semaphore here and block the cmd buffer submission on it
                    let fence = device.create_fence(&FenceCreateInfo::builder(), None)
                        .expect("failed to create fence");
                    let (index, is_suboptimal) = globals.device.swapchain_loader.acquire_next_image(
                        swapchain_surface.swapchain, u64::MAX, Semaphore::null(), fence)
                        .expect("failed to acquire swapchain image");
                    device.wait_for_fences(&[fence], true, u64::MAX)
                        .expect("failed to wait for fences");
                    if is_suboptimal {
                        self.recreate_swapchain();
                    }
                    if let Surface::Swapchain(swapchain_surface) = self { // avoid two borrows
                        (swapchain_surface.swapchain_images[index as usize], index)
                    } else {
                        panic!()
                    }
                },
            };

            // for each image {

            //let image: GpuImage;
            //if image.image_buf.descriptior_set.as_raw() == 0 {
            //    let new_set = globals.images_descriptor_set_allocator.allocate();
            //    let image_info = &[
            //        DescriptorImageInfo::builder()
            //            .image_view(image.image_buf.vk_image_view)
            //            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            //            .build()
            //    ];
            //    let descriptor_writes = &[WriteDescriptorSet::builder()
            //        .dst_set(new_set)
            //        .dst_binding(0)
            //        .descriptor_type(DescriptorType::STORAGE_IMAGE)
            //        .image_info(image_info)
            //        .build()];
            //        device.update_descriptor_sets(descriptor_writes, &[]);
            //    image.image_buf.descriptior_set = new_set;
            //}
            //let descriptor_sets = &[image.image_buf.descriptior_set];
            //device.cmd_bind_descriptor_sets(
            //    cmd_buffer, PipelineBindPoint::GRAPHICS, pipeline_layout, 1, descriptor_sets, &[]);
            //    device.cmd_draw(cmd_buffer, 4, 1, 0, 0);

            // }

            let graphics_queue = device.get_device_queue(
                globals.device.queue_family_indices.graphics, 0);
            let cmd_buffers = &[cmd_buffer];
            let submit_infos = &[
                SubmitInfo::builder().command_buffers(cmd_buffers).build()
            ];
            let fence = Fence::null(); // TODO: use fence
            device.queue_submit(graphics_queue, submit_infos, fence)
                .expect("failed to submit command buffer");
            device.queue_wait_idle(graphics_queue)
                .expect("failed to wait for graphics queue idle");

            match self {
                Surface::Image(_, _) => {},
                Surface::Swapchain(swapchain_surface) => {
                    let present_queue = device.get_device_queue(
                        globals.device.queue_family_indices.present, 0);
                    globals.device.swapchain_loader.queue_present(present_queue,
                        &PresentInfoKHR::builder()
                            .swapchains(&[swapchain_surface.swapchain])
                            .image_indices(&[dst_image_index]))
                        .expect("failed to present");
                    device.queue_wait_idle(present_queue)
                        .expect("failed to wait for present queue idle");
                    println!("presented");
                },
            }
        }

    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            match self {
                Surface::Image(_, _) => {},
                Surface::Swapchain(fields) => {
                    let globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();
                    // TODO: wait until drawing finished
                    globals.device.swapchain_loader.destroy_swapchain(fields.swapchain, None);
                    globals.surface_loader.destroy_surface(fields.vulkan_surface, None);
                },
            }
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
