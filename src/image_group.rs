
use ash::vk::DescriptorSet;

pub(crate) const IMAGE_GROUP_SIZE: u32 = 4;
//pub(crate) const IMAGE_GROUP_SIZE: u32 = 96;

struct ImageGroup {
    //sets: Vec<DescriptorSet>,
}

impl ImageGroup {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn add_image(&self) {
    }
}
