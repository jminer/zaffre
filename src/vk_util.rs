
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::{MutexGuard, Arc};

use ash::{Entry, Instance};
use ash::vk::*;
use ash::extensions::khr::{Surface, Swapchain, Win32Surface};
use ash::extensions::ext::DebugUtils;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

use smallvec::SmallVec;

use crate::vk_descriptor_set_allocator::DescriptorSetAllocator;
use crate::image_group::IMAGE_GROUP_SIZE;
use crate::retained::{ImageStatus, VULKAN_GLOBALS, ImageCopyOp};
use crate::vk_allocator::Allocator;

// TODO: I should probably put this or a type that does the same thing into a crate, since I've
// copied it into four projects now (clear-coat, nightshade?, radiance, zaffre).
pub(crate) fn str_to_c_vec<'a: 'b, 'b, A: ::smallvec::Array<Item=u8>>(s: &'a str, buf: &'b mut SmallVec<A>) -> *const c_char {
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
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

pub struct VulkanGlobals {
    pub entry: Entry,
    pub instance: Arc<Instance>,
    pub debug_messenger: Option<DebugUtilsMessengerEXT>,
    pub surface_loader: Surface,
    pub device: VulkanDevice,
    pub(crate) allocator: Allocator,
    pub(crate) samplers_descriptor_set_allocator: DescriptorSetAllocator,
    pub(crate) images_descriptor_set_allocator: DescriptorSetAllocator,
    pub(crate) transfer_command_pool: CommandPool,
    pub(crate) graphics_command_pool: CommandPool,
    pub(crate) pipelines: HashMap<PipelineArgs, (Pipeline, PipelineLayout)>,
    pub(crate) ops: Vec<Arc<ImageCopyOp>>,
}

pub struct VulkanDevice {
    // Since images are only useable with the device they were created for, it would be too
    // complicated to try to have multiple logical devices. Also, an app can create an image before
    // a swapchain, so we won't know what Vulkan surface we need to have present support for until
    // after we've already picked a physical device and created a logical device. However, looking
    // at every platform in the Vulkan spec, it looks like if a device can present to one surface,
    // it can present to any surface, because of the existence of
    // vkGetPhysicalDeviceWin32PresentationSupportKHR and the like (and I can't imagine it working
    // differently). So we can just use vkGetPhysicalDeviceWin32PresentationSupportKHR,
    // vkGetPhysicalDeviceWaylandPresentationSupportKHR, etc.
    pub physical: PhysicalDevice,
    pub queue_family_indices: QueueFamilyIndices,
    pub logical: Arc<ash::Device>,
    pub swapchain_loader: Swapchain,
}

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
    pub transfer: u32,
}

impl QueueFamilyIndices {
    fn new(graphics: u32, present: u32, transfer: u32) -> QueueFamilyIndices {
        // TODO: fill in unique indices here?
        Self { graphics, present, transfer }
    }

    pub fn get_unique_indices(&self) -> Vec<u32> {
        let mut indices = Vec::with_capacity(3);
        indices.push(self.graphics);
        if !indices.contains(&self.present) {
            indices.push(self.present);
        }
        if !indices.contains(&self.transfer) {
            indices.push(self.transfer);
        }
        indices
    }
}

pub(crate) fn create_instance() -> VulkanGlobals {
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
        let layers: Vec<*const c_char> = get_instance_layer_list(&entry);
        let (extensions, debug_utils_enabled): (Vec<*const c_char>, _) =
            get_instance_extensions_list(&entry);
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
        let instance = Arc::new(entry.create_instance(&create_info, None)
            .expect("failed to create instance"));


        let debug_messenger = if debug_utils_enabled {
            Some(create_debug_messenger(&entry, &instance))
        } else {
            None
        };
        let surface_loader = Surface::new(&entry, &*instance);
        let device = create_device(&entry, &instance);


        let graphics_command_pool = device.logical.create_command_pool(
            &CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_indices.graphics),
            None)
            .expect("failed to create graphics command pool");
        let transfer_command_pool = device.logical.create_command_pool(
            &CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_indices.transfer),
            None)
            .expect("failed to create transfer command pool");

        let samplers_desc_set_layout_bindings = &[
            DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(DescriptorType::SAMPLER)
                .descriptor_count(3)
                .stage_flags(ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let samplers_descriptor_set_allocator = DescriptorSetAllocator::new(device.logical.clone(), DescriptorSetLayoutCreateFlags::empty(), samplers_desc_set_layout_bindings, 1);

        let images_desc_set_layout_bindings = &[
            DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let images_descriptor_set_allocator = DescriptorSetAllocator::new(device.logical.clone(), DescriptorSetLayoutCreateFlags::empty(), images_desc_set_layout_bindings, 16);
        let device_physical = device.physical;
        let device_logical = device.logical.clone();

        VulkanGlobals {
            entry,
            instance: instance.clone(),
            debug_messenger,
            surface_loader,
            device,
            allocator: Allocator {
                instance: instance,
                phy_device: device_physical,
                device: device_logical,
                large_allocation_threshold: 1024 * 1024,
                chunk_size: 4 * 1024 * 1024,
                mem_type_chunks: Vec::new(),
            },
            transfer_command_pool,
            graphics_command_pool,
            samplers_descriptor_set_allocator,
            images_descriptor_set_allocator,
            pipelines: HashMap::new(),
            ops: Vec::new(),
        }
    }
}

fn get_instance_layer_list(entry: &Entry) -> Vec<*const c_char> {
    // I was seeing VK_LAYER_LUNARG_standard_validation in the list when I hadn't installed the
    // Vulkan SDK. I had downloaded the SDK a few months prior, and I thought I had installed it
    // because the layer was showing up in the list. But after not getting any debug messages even
    // when I purposfully passed invalid arguments, I checked and found I hadn't installed the SDK.
    let mut layers = vec![];
    println!("Vulkan instance layers:");
    let layer_props = entry.enumerate_instance_layer_properties()
        .expect("failed to get instance layer properties");
    for layer_prop in layer_props {
        let layer_name = unsafe { CStr::from_ptr(layer_prop.layer_name.as_ptr()) };
        println!("  {}", layer_name.to_string_lossy());
        if cfg!(debug_assertions) {
            if layer_name == STANDARD_VALIDATION_LAYER_NAME {
                layers.push(STANDARD_VALIDATION_LAYER_NAME);
            }
        }
    }
    layers.iter().map(|ext| ext.as_ptr()).collect()
}

#[cfg(windows)]
fn native_surface_ext() -> &'static CStr {
    Win32Surface::name()
}

fn get_instance_extensions_list(entry: &Entry) -> (Vec<*const c_char>, bool) {
    let mut extensions = vec![
        Surface::name(), native_surface_ext()
    ];
    let mut debug_utils_enabled = false;
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
                debug_utils_enabled = true;
            }
        }
    }
    (extensions.iter().map(|ext| ext.as_ptr()).collect(), debug_utils_enabled)
}

pub(crate) unsafe fn get_device_extensions_list(
    instance: &Instance,
    phy_device: PhysicalDevice,
) -> Vec<*const c_char> {
    let extensions = vec![
        Swapchain::name(),
    ];
    println!("Vulkan device extensions:");
    let ext_props = instance.enumerate_device_extension_properties(phy_device)
        .expect("failed to get extension properties");
    for ext_prop in ext_props {
        let ext_name = CStr::from_ptr(ext_prop.extension_name.as_ptr());
        println!("  {}", ext_name.to_string_lossy());
    }
    extensions.iter().map(|ext| ext.as_ptr()).collect()
}

unsafe fn create_debug_messenger(entry: &Entry, instance: &Instance) -> DebugUtilsMessengerEXT {
    let debug_utils_loader = DebugUtils::new(entry, instance);

    let severity = DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
        DebugUtilsMessageSeverityFlagsEXT::WARNING |
        DebugUtilsMessageSeverityFlagsEXT::ERROR;
    let ty = DebugUtilsMessageTypeFlagsEXT::GENERAL |
        DebugUtilsMessageTypeFlagsEXT::VALIDATION |
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
    let create_info = DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(severity)
        .message_type(ty)
        .pfn_user_callback(Some(debug_callback));
    debug_utils_loader.create_debug_utils_messenger(&create_info, None)
        .expect("failed to create debug messenger")
}

unsafe extern "system" fn debug_callback(
    _message_severity: DebugUtilsMessageSeverityFlagsEXT,
    _message_types: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("validation msg: {}", message.to_string_lossy());
    FALSE
}

unsafe fn create_device(
    entry: &Entry,
    instance: &Instance,
) -> VulkanDevice {
    let (phy_device, indices) = get_preferred_physical_device(entry, instance)
        .expect("no acceptable physical device found");

    const QUEUE_PRIORITIES: &[f32] = &[1.0]; // only create one queue
    let queue_create_infos: Vec<_> = indices.get_unique_indices().iter().map(|i|
        DeviceQueueCreateInfo::builder()
            .queue_family_index(*i)
            .queue_priorities(QUEUE_PRIORITIES) // sets queue count too
            .build()
    ).collect();
    let extensions = get_device_extensions_list(&instance, phy_device);
    let enabled_features = PhysicalDeviceFeatures::builder().build();
    let device_create_info = DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&extensions)
        .enabled_features(&enabled_features);
    let device = Arc::new(instance.create_device(phy_device, &device_create_info, None)
        .expect("failed to create logical device"));
    let swapchain_loader = Swapchain::new(instance, &*device);

    VulkanDevice {
        physical: phy_device,
        queue_family_indices: indices,
        logical: device,
        swapchain_loader,
    }
}

unsafe fn get_preferred_physical_device(
    entry: &Entry,
    instance: &Instance,
) -> Option<(PhysicalDevice, QueueFamilyIndices)> {
    let phy_devices = instance.enumerate_physical_devices()
        .expect("failed to get physical devices");
    let filtered_devices: Vec<_> = phy_devices.iter().filter_map(|dev| {
        // vulkan-tutorial.com checks if queueCount > 0, but the Vulkan spec says "Each queue
        // family must support at least one queue." so I don't think that is necessary.
        let queue_family_props = instance
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
        // Find any queue that supports presentation.
        let present_index = (0..queue_family_props.len() as u32)
            .find(|i| get_physical_device_presentation_support(entry, instance, *dev, *i));
        if let (Some(graphics_index), Some(present_index), Some(transfer_index)) =
            (graphics_index, present_index, transfer_index)
        {
            Some((*dev, QueueFamilyIndices {
                graphics: graphics_index as u32,
                present: present_index as u32,
                transfer: transfer_index as u32,
            }))
        } else {
            None
        }
    }).collect();
    for (dev, queue_family_indices) in filtered_devices.iter() {
        let props = instance.get_physical_device_properties(*dev);
        if props.device_type == PhysicalDeviceType::DISCRETE_GPU {
            return Some((*dev, *queue_family_indices));
        }
    }
    if !filtered_devices.is_empty() {
        return Some(filtered_devices[0]);
    }
    None
}

#[cfg(windows)]
unsafe fn get_physical_device_presentation_support(
    entry: &Entry,
    instance: &Instance,
    physical_device: PhysicalDevice,
    queue_family_index: u32,
) -> bool {
    // temporary until it is added to ash
    let surface_fn = KhrWin32SurfaceFn::load(|name|
        mem::transmute(entry.get_instance_proc_addr(instance.handle(), name.as_ptr()))
    );
    surface_fn.get_physical_device_win32_presentation_support_khr(
        physical_device, queue_family_index
    ) != 0
}

macro_rules! include_shader {
    ($file:expr) => {
        {
            let bytes = include_bytes!($file);
            unsafe { mem::transmute(*bytes) }
        }
    };
}

const VERT_SHADER_SIZE: usize = include_bytes!("../target/shaders/fill.frag.spv").len();
const VERT_SHADER: &[u32; VERT_SHADER_SIZE / 4] =
    &include_shader!("../target/shaders/fill.frag.spv");// TODO: fix name

const FRAG_SHADER_SIZE: usize = include_bytes!("../target/shaders/fill.frag.spv").len();
const FRAG_SHADER: &[u32; FRAG_SHADER_SIZE / 4] =
    &include_shader!("../target/shaders/fill.frag.spv");


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineArgs {
    pub(crate) color_attachment_format: Format,
    // Maybe I can set this to PRESENT_SRC for swapchain images, to GENERAL for host visible images,
    // and SHADER_READ_ONLY for GPU images?
    pub(crate) color_attachment_final_layout: ImageLayout,
    pub(crate) polygon_mode: PolygonMode,
}

pub(crate) unsafe fn get_pipeline(device: &ash::Device, args: PipelineArgs)
    -> (Pipeline, PipelineLayout)
{
    let mut globals: MutexGuard<VulkanGlobals> = VULKAN_GLOBALS.lock().unwrap();

    return *globals.pipelines.entry(args).or_insert_with(
        || create_pipeline(device, args));
}

unsafe fn create_pipeline(device: &ash::Device, args: PipelineArgs) -> (Pipeline, PipelineLayout) {
    let vertex_shader_module = device.create_shader_module(
        &ShaderModuleCreateInfo::builder().code(VERT_SHADER), None
    ).expect("failed to create vert shader module");

    let fragment_shader_module = device.create_shader_module(
        &ShaderModuleCreateInfo::builder().code(FRAG_SHADER), None
    ).expect("failed to create frag shader module");

    let stages = &[
        PipelineShaderStageCreateInfo {
            stage: ShaderStageFlags::VERTEX,
            module: vertex_shader_module,
            p_name: "main\0".as_ptr() as *const c_char,
            ..Default::default()
        },
        PipelineShaderStageCreateInfo {
            stage: ShaderStageFlags::FRAGMENT,
            module: fragment_shader_module,
            p_name: "main\0".as_ptr() as *const c_char,
            ..Default::default()
        },
    ];

    let vertex_input = PipelineVertexInputStateCreateInfo::builder();

    let input_assembly = PipelineInputAssemblyStateCreateInfo::builder()
        .topology(PrimitiveTopology::TRIANGLE_LIST);

    let viewport = Viewport::default();

    let scissor = Rect2D::default();

    let viewport_state = PipelineViewportStateCreateInfo {
        viewport_count: 1,
        p_viewports: &viewport as *const Viewport,
        scissor_count: 1,
        p_scissors: &scissor as *const Rect2D,
        ..Default::default()
    };

    let rasterization = PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(PolygonMode::FILL) // TODO: check for and enable GPU feature?
        .cull_mode(CullModeFlags::BACK)
        .front_face(FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1.0);

    let multisample = PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(SampleCountFlags::TYPE_1)
        .sample_shading_enable(false);

    let color_blend_attachments = &[
        PipelineColorBlendAttachmentState::builder()
        .blend_enable(true)
        // These factors are for premultiplied textures and colors.
        .src_color_blend_factor(BlendFactor::ONE)
        .dst_color_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
        .src_alpha_blend_factor(BlendFactor::ONE)
        .dst_alpha_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
        .build()
    ];

    let color_blend_state = PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(color_blend_attachments);

    let dynamic_state_array = &[DynamicState::VIEWPORT, DynamicState::SCISSOR];
    let dynamic_state = PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(dynamic_state_array);

    let pipeline_layout_create_info = PipelineLayoutCreateInfo::builder();
    // TODO: descriptor sets
    let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_create_info, None)
        .expect("failed to create pipeline layout");

    // TODO: It doesn't make much sense to create a render pass in this create_pipeline function
    //       because if a render pass has multiple subpasses, it will usually use a different
    //       pipeline in each subpass.
    let attachment_descriptions = &[
        AttachmentDescription::builder()
        .format(args.color_attachment_format)
        .samples(SampleCountFlags::TYPE_1)
        .load_op(AttachmentLoadOp::LOAD) // TODO: needs to be configuable?
        .store_op(AttachmentStoreOp::STORE)
        .stencil_load_op(AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(AttachmentStoreOp::DONT_CARE)
        .initial_layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .final_layout(args.color_attachment_final_layout)
        .build()
    ];

    let color_attachment_refs = &[
        AttachmentReference::builder()
        .attachment(0)
        .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build()
    ];

    let subpass_descriptions = &[
        SubpassDescription::builder()
        .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachment_refs)
        .build()
    ];

    let render_pass_create_info = RenderPassCreateInfo::builder()
        .attachments(attachment_descriptions)
        .subpasses(subpass_descriptions);
    let render_pass = device.create_render_pass(&render_pass_create_info, None)
        .expect("failed to create render pass");

    let pipeline_create_info = GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .color_blend_state(&color_blend_state)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();

    // TODO: use pipeline cache?
    let pipeline =
        device.create_graphics_pipelines(PipelineCache::null(), &[pipeline_create_info], None)
            .expect("failed to create pipeline")[0];

    device.destroy_shader_module(vertex_shader_module, None);
    device.destroy_shader_module(fragment_shader_module, None);

    (pipeline, pipeline_layout)
}
