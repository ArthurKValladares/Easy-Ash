use crate::{device::Device, util};
use anyhow::Result;
use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferCreationError {
    #[error("could not find memory index for buffer")]
    CouldNotFindMemoryIndex,
}

pub enum BufferType {
    Index,
    Storage,
}

impl BufferType {
    fn usage(&self) -> vk::BufferUsageFlags {
        match self {
            BufferType::Index => vk::BufferUsageFlags::INDEX_BUFFER,
            BufferType::Storage => todo!(),
        }
    }
}

pub struct Buffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn with_size(device: &Device, size: u64, ty: BufferType) -> Result<Self> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(ty.usage())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.device.create_buffer(&buffer_info, None)? };
        let index_buffer_memory_req =
            unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let buffer_memory_index = util::find_memory_type_index(
            &index_buffer_memory_req,
            &device.memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT, // TODO: Configurable somehow, abstracted away
        )
        .ok_or(BufferCreationError::CouldNotFindMemoryIndex)?;

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: index_buffer_memory_req.size,
            memory_type_index: buffer_memory_index,
            ..Default::default()
        };
        let memory = unsafe { device.device.allocate_memory(&allocate_info, None)? };

        Ok(Self { buffer, memory })
    }
}
