use crate::{device::Device, resources::buffer::Buffer};
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

#[derive(Debug, Copy, Clone)]
pub enum DescriptorType {
    StorageBuffer(DescriptorBufferInfo),
    UniformBuffer(DescriptorBufferInfo),
}

impl From<DescriptorType> for vk::DescriptorType {
    fn from(ty: DescriptorType) -> Self {
        match ty {
            DescriptorType::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
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

#[derive(Debug, Copy, Clone)]
pub struct BindingDesc {
    ty: DescriptorType,
    count: u32,
    stage: ShaderStage,
}

impl BindingDesc {
    pub fn new(ty: DescriptorType, count: u32, stage: ShaderStage) -> Self {
        Self { ty, count, stage }
    }
}

pub struct DescriptorSet {
    pub layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
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
                    .descriptor_type(desc.ty.into())
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

        let write_descriptor_sets = binding_descs
            .iter()
            .enumerate()
            .map(|(idx, desc)| {
                // TODO: Need to hook up buffer pointer
                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(idx as u32)
                    .descriptor_type(desc.ty.into());
                match &desc.ty {
                    DescriptorType::StorageBuffer(info) => {
                        write.buffer_info(std::slice::from_ref(&info.info))
                    }
                    DescriptorType::UniformBuffer(info) => {
                        write.buffer_info(std::slice::from_ref(&info.info))
                    }
                }
                .build()
            })
            .collect::<Vec<_>>();

        Ok(Self {
            layout,
            descriptor_set,
            write_descriptor_sets,
        })
    }

    pub fn update(&self, device: &Device) {
        // TODO: Should have a way to update several sets together
        unsafe {
            device
                .device
                .update_descriptor_sets(&self.write_descriptor_sets, &[])
        };
    }
}
