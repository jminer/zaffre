use std::sync::Arc;
use ash::version::DeviceV1_0;
use ash::vk::*;

use crate::AHashMap;

pub(crate) struct DescriptorSetAllocator {
    device: Arc<ash::Device>,
    layout: DescriptorSetLayout,
    pool_size: u32,
    pool_sizes: Vec<DescriptorPoolSize>,
    pools: Vec<(u32, DescriptorPool)>, // (count of sets allocated from pool, pool)
    pool_map: AHashMap<DescriptorSet, usize>, // maps the descriptor set to a pool index
}

impl DescriptorSetAllocator {
    pub(crate) unsafe fn new(
        device: Arc<ash::Device>,
        layout_create_flags: DescriptorSetLayoutCreateFlags,
        layout_bindings: &[DescriptorSetLayoutBinding],
        pool_size: u32,
    ) -> Self {
        let layout_create_info = DescriptorSetLayoutCreateInfo::builder()
            .flags(layout_create_flags)
            .bindings(layout_bindings);
        let layout = device.create_descriptor_set_layout(&layout_create_info, None)
            .expect("failed to create descriptor set layout");
        let pool_sizes: Vec<_> = layout_bindings.iter().map(|binding| DescriptorPoolSize {
            ty: binding.descriptor_type,
            descriptor_count: binding.descriptor_count,
        }).collect();
        let pools = Vec::new();
        Self {
            device,
            layout,
            pool_size,
            pool_sizes,
            pools,
            pool_map: Default::default(),
        }
    }

    pub(crate) fn layout(&self) -> DescriptorSetLayout {
        self.layout
    }

    fn add_pool(&mut self) {
        unsafe {
            let create_info = DescriptorPoolCreateInfo::builder()
                .flags(DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
                .max_sets(self.pool_size)
                .pool_sizes(&self.pool_sizes);
            self.pools.push((0, self.device.create_descriptor_pool(&create_info, None)
                .expect("failed to create descriptor pool")));
        }
    }

    pub(crate) fn allocate(&mut self) -> DescriptorSet {
        unsafe {
            let pool_index = self.pools.iter()
                .position(|(count, _)| *count < self.pool_size)
                .unwrap_or_else(|| {
                    self.add_pool();
                    self.pools.len() - 1
                });
            let pool_entry = &mut self.pools[pool_index];

            let layouts = &[self.layout];
            let allocate_info = DescriptorSetAllocateInfo::builder()
                .descriptor_pool(pool_entry.1)
                .set_layouts(layouts);
            let set = self.device.allocate_descriptor_sets(&allocate_info)
                .expect("failed to allocate descriptor set")[0];
            self.pool_map.insert(set, pool_index);
            set
        }
    }

    pub(crate) unsafe fn free(&mut self, set: DescriptorSet) {
        let pool_index = self.pool_map.remove(&set)
            .expect("descriptor set not found in pool map");
        let pool_entry = &mut self.pools[pool_index];
        self.device.free_descriptor_sets(pool_entry.1, &[set]);
        pool_entry.0 -= 1;
        // To free a descriptor pool, it would be easy to free it, then loop over the pool_map and
        // decrement any higher pool_index. I don't think it is needed though.
    }
}

impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_descriptor_set_layout(self.layout, None);
            for pool_entry in self.pools.iter() {
                self.device.destroy_descriptor_pool(pool_entry.1, None);
            }
        }
    }
}
