use ash::vk;

pub fn find_memory_type_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}

pub unsafe fn mem_copy<T: Copy>(ptr: *mut std::ffi::c_void, data: &[T]) {
    let elem_size = std::mem::size_of::<T>() as vk::DeviceSize;
    let size = data.len() as vk::DeviceSize * elem_size;
    let mut align = ash::util::Align::new(ptr, elem_size, size);
    align.copy_from_slice(data);
}

pub struct MemoryMappablePointer(*mut std::ffi::c_void);
unsafe impl Send for MemoryMappablePointer {}
unsafe impl Sync for MemoryMappablePointer {}

impl MemoryMappablePointer {
    pub unsafe fn from_raw_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    pub fn mem_copy<T: Copy>(&self, data: &[T]) {
        unsafe { mem_copy(self.0, data) };
    }
}


// TODO: Move
pub fn size_of_slice<T>(data: &[T]) -> u64 {
    std::mem::size_of_val(data) as u64
}

pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
    std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
    }
}