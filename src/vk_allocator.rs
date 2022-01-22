
use std::sync::Arc;

use ash::vk::*;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DeviceMemoryRef {
    pub(crate) memory: DeviceMemory,
    pub(crate) offset: u64,
}

pub(crate) struct Allocator {
    pub(crate) instance: Arc<ash::Instance>,
    pub(crate) phy_device: PhysicalDevice,
    pub(crate) device: Arc<ash::Device>,
    // Any requested allocation larger than this is allocated directly from the driver instead of
    // from a chunk.
    pub(crate) large_allocation_threshold: u64,
    pub(crate) chunk_size: u64,
    // There should only be one MemoryTypeChunks for each memory type.
    pub(crate) mem_type_chunks: Vec<MemoryTypeChunks>,
}

impl Allocator {
    pub(crate) unsafe fn allocate(
        &self,
        mem_requirements: MemoryRequirements,
        memory_properties: MemoryPropertyFlags,
    ) -> DeviceMemoryRef {
        // TODO: only do this for allocations > 2MB or something
        // Smaller allocations should actually share a larger driver allocation.
        let mem_type = self.find_memory_type(mem_requirements.memory_type_bits, memory_properties);
        let mem_allocate_info = MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type);
        let memory = self.device.allocate_memory(&mem_allocate_info, None)
            .expect("failed to allocate memory");
        DeviceMemoryRef {
            memory,
            offset: 0,
        }
    }

    pub(crate) unsafe fn free(&self, mem: DeviceMemoryRef) {
        self.device.free_memory(mem.memory, None);
    }

    // Returns the index of the first memory type found that is allowed by `filter_mask` and has
    // the properties `memory_properties`.
    fn find_memory_type(&self, filter_mask: u32, memory_properties: MemoryPropertyFlags) -> u32 {
        let props = unsafe { self.instance.get_physical_device_memory_properties(self.phy_device) };
        for i in 0..props.memory_type_count {
            if (1 << i) & filter_mask == 1 &&
                props.memory_types[i as usize].property_flags.contains(memory_properties)
            {
                return i;
            }
        }

        panic!("failed to find acceptable memory type");
    }
}

pub(crate) struct MemoryTypeChunks {
    memory_type_index: u32,
    memory_properties: MemoryPropertyFlags,
    chunks: Vec<Chunk>,
}

struct Chunk {
    memory: DeviceMemory,
    size: u64,

    // TODO: metadata on what's allocated, etc.
}
