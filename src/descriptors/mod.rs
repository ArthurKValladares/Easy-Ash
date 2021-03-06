use crate::{
    device::Device,
    resources::{ash_image::Image, buffer::Buffer, sampler::Sampler},
};
use anyhow::Result;
use ash::vk;

const MAX_BINDLESS_RESOURCES: u32 = 16536;

pub struct DescriptorPool {
    pool: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn new(device: &Device) -> Result<Self> {
        let descriptor_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: MAX_BINDLESS_RESOURCES,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: MAX_BINDLESS_RESOURCES,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND_EXT)
            .max_sets(MAX_BINDLESS_RESOURCES * descriptor_sizes.len() as u32)
            .pool_sizes(&descriptor_sizes)
            .max_sets(descriptor_sizes.len() as u32);

        let pool = unsafe {
            device
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)?
        };

        Ok(Self { pool })
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_descriptor_pool(self.pool, None);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DescriptorBufferInfo {
    info: vk::DescriptorBufferInfo,
}

impl DescriptorBufferInfo {
    pub fn new(buffer: &Buffer, offset: Option<u64>, range: Option<u64>) -> Self {
        let info = vk::DescriptorBufferInfo {
            buffer: buffer.buffer,
            offset: offset.unwrap_or(0),
            range: range.unwrap_or(buffer.size),
        };
        Self { info }
    }
}

pub fn new_descriptor_image_info(image: &Image, sampler: &Sampler) -> vk::DescriptorImageInfo {
    vk::DescriptorImageInfo {
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        image_view: image.view,
        sampler: sampler.sampler,
    }
}

// TODO: Theres' some duplicated state in here and `DescriptorInfo`, get rid of it later
#[derive(Debug, Clone)]
pub enum DescriptorType {
    StorageBuffer,
    UniformBuffer,
    CombinedImageSampler,
}

impl From<DescriptorType> for vk::DescriptorType {
    fn from(ty: DescriptorType) -> Self {
        match ty {
            DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }
}

impl From<&DescriptorType> for vk::DescriptorType {
    fn from(ty: &DescriptorType) -> Self {
        match ty {
            DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }
}

pub enum DescriptorInfo {
    StorageBuffer(DescriptorBufferInfo),
    UniformBuffer(DescriptorBufferInfo),
    CombinedImageSampler(Vec<vk::DescriptorImageInfo>),
}

impl From<DescriptorInfo> for vk::DescriptorType {
    fn from(info: DescriptorInfo) -> Self {
        match info {
            DescriptorInfo::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorInfo::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorInfo::CombinedImageSampler(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }
}

impl From<&DescriptorInfo> for vk::DescriptorType {
    fn from(info: &DescriptorInfo) -> Self {
        match info {
            DescriptorInfo::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorInfo::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorInfo::CombinedImageSampler(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl From<ShaderStage> for vk::ShaderStageFlags {
    fn from(stage: ShaderStage) -> Self {
        match stage {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindingDesc {
    ty: DescriptorType,
    count: u32,
    stage: ShaderStage,
}

impl BindingDesc {
    pub fn new(ty: DescriptorType, count: u32, stage: ShaderStage) -> Self {
        Self { ty, count, stage }
    }

    pub fn ty(&self) -> &DescriptorType {
        &self.ty
    }
}

pub struct DescriptorSet {
    pub layout: vk::DescriptorSetLayout,
    pub descriptor_set: vk::DescriptorSet,
    write_descriptor_sets: Vec<vk::WriteDescriptorSet>,
}

impl DescriptorSet {
    pub fn new(
        device: &Device,
        descriptor_pool: &DescriptorPool,
        binding_descs: &[BindingDesc],
    ) -> Result<Self> {
        let bindings = binding_descs
            .iter()
            .enumerate()
            .map(|(idx, desc)| {
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(idx as u32)
                    .descriptor_type(desc.ty().into())
                    .descriptor_count(desc.count)
                    .stage_flags(desc.stage.into())
                    .build()
            })
            .collect::<Vec<_>>();

        let descriptor_set_layout_ci = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .build();
        let layout = unsafe {
            device
                .device
                .create_descriptor_set_layout(&descriptor_set_layout_ci, None)?
        };
        let descriptor_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.pool)
            .set_layouts(std::slice::from_ref(&layout));
        let descriptor_set = unsafe {
            device
                .device
                .allocate_descriptor_sets(&descriptor_alloc_info)?[0]
        };

        Ok(Self {
            layout,
            descriptor_set,
            write_descriptor_sets: Default::default(),
        })
    }

    pub fn bind(&mut self, infos: &[DescriptorInfo]) {
        self.write_descriptor_sets = infos
            .iter()
            .enumerate()
            .map(|(idx, info)| {
                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(self.descriptor_set)
                    .dst_binding(idx as u32)
                    .descriptor_type(info.into());
                match info {
                    DescriptorInfo::StorageBuffer(info) => {
                        write.buffer_info(std::slice::from_ref(&info.info))
                    }
                    DescriptorInfo::UniformBuffer(info) => {
                        write.buffer_info(std::slice::from_ref(&info.info))
                    }
                    DescriptorInfo::CombinedImageSampler(infos) => write.image_info(infos),
                }
                .build()
            })
            .collect::<Vec<_>>();
    }

    pub fn update(&self, device: &Device) {
        // TODO: Should have a way to update several sets together
        unsafe {
            device
                .device
                .update_descriptor_sets(&self.write_descriptor_sets, &[])
        };
    }

    pub unsafe fn clean(&self, device: &Device) {
        device
            .device
            .destroy_descriptor_set_layout(self.layout, None);
    }
}
