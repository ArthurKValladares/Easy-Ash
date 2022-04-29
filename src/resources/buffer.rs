use crate::{
    device::Device,
    mem::{self, MemoryMappablePointer},
};
use anyhow::Result;
use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BufferCreationError {
    #[error("could not find memory index for buffer")]
    CouldNotFindMemoryIndex,
}

#[derive(Debug, Error)]
pub enum MemoryCopyError {
    #[error("buffer does not allow CPU -> GPU transfer")]
    CannotTransferFromCPU,
}

pub enum BufferType {
    Index,
    Storage,
}

impl BufferType {
    fn usage(&self) -> vk::BufferUsageFlags {
        match self {
            BufferType::Index => vk::BufferUsageFlags::INDEX_BUFFER,
            BufferType::Storage => vk::BufferUsageFlags::STORAGE_BUFFER,
        }
    }
}

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub size: u64,
    memory: vk::DeviceMemory,
    ptr: Option<MemoryMappablePointer>,
}

impl Buffer {
    pub fn with_size(device: &Device, size: u64, ty: BufferType) -> Result<Self> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(ty.usage())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.device.create_buffer(&buffer_info, None)? };
        let buffer_memory_req = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        // TODO: Configurable somehow, abstracted away
        let flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let buffer_memory_index =
            mem::find_memory_type_index(&buffer_memory_req, &device.memory_properties, flags)
                .ok_or(BufferCreationError::CouldNotFindMemoryIndex)?;

        let allocate_info = vk::MemoryAllocateInfo {
            allocation_size: buffer_memory_req.size,
            memory_type_index: buffer_memory_index,
            ..Default::default()
        };
        let memory = unsafe { device.device.allocate_memory(&allocate_info, None)? };
        // TODO: offset at some point. Should I also bind this unconditionally?
        unsafe { device.device.bind_buffer_memory(buffer, memory, 0)? };
        let ptr = if flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
            unsafe {
                let ptr = device.device.map_memory(
                    memory,
                    0,
                    buffer_memory_req.size,
                    vk::MemoryMapFlags::empty(),
                )?;
                Some(MemoryMappablePointer::from_raw_ptr(ptr))
            }
        } else {
            None
        };

        Ok(Self {
            buffer,
            size,
            memory,
            ptr,
        })
    }

    pub fn from_data<T: Copy>(device: &Device, ty: BufferType, data: &[T]) -> Result<Self> {
        let size = mem::size_of_slice(data);
        let buffer = Self::with_size(device, size, ty)?;
        buffer.copy_data(data)?;
        Ok(buffer)
    }

    pub fn copy_data<T: Copy>(&self, data: &[T]) -> Result<(), MemoryCopyError> {
        if let Some(ptr) = &self.ptr {
            ptr.mem_copy(data);
            Ok(())
        } else {
            Err(MemoryCopyError::CannotTransferFromCPU)
        }
    }

    pub unsafe fn clean(&self, device: &Device) {
        device.device.destroy_buffer(self.buffer, None);
        device.device.free_memory(self.memory, None);
    }
}
