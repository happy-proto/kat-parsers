//! 预生成 parser 的共享解压运行时；不依赖 Tree-sitter 的私有布局。

/// 解压为四字节对齐、使用目标原生字节序的表，内存与进程同寿命。
///
/// # Safety
/// `input` 必须指向 `input_len` 个可读字节；长度与分界必须来自同一次生成。
/// 调用方必须用每语言的 `OnceLock` 保证只初始化一次，并同步发布语言指针。
pub unsafe extern "C" fn decode(
    input: *const u8,
    input_len: usize,
    output_len: usize,
    map_offset: usize,
) -> *const u8 {
    if input.is_null()
        || output_len == 0
        || !output_len.is_multiple_of(4)
        || !map_offset.is_multiple_of(4)
        || map_offset >= output_len
    {
        std::process::abort();
    }
    let mut words = vec![0_u32; output_len / 4].into_boxed_slice();
    let output =
        unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr().cast::<u8>(), output_len) };
    let input = unsafe { std::slice::from_raw_parts(input, input_len) };
    if !matches!(zstd::bulk::decompress_to_buffer(input, output), Ok(size) if size == output_len) {
        std::process::abort();
    }
    // 文件固定为小端；uint16 表和 uint32 map 在大端目标上分别转换。
    if cfg!(target_endian = "big") {
        for pair in output[..map_offset].as_chunks_mut::<2>().0 {
            pair.reverse();
        }
        for word in output[map_offset..].as_chunks_mut::<4>().0 {
            word.reverse();
        }
    }
    Box::into_raw(words).cast::<u32>().cast::<u8>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_uses_aligned_allocation_and_typed_endian_boundaries() {
        let input = [0x34, 0x12, 0x78, 0x56, 0xef, 0xcd, 0xab, 0x90];
        let packed = zstd::bulk::compress(&input, 19).unwrap();
        let pointer = unsafe { decode(packed.as_ptr(), packed.len(), input.len(), 4) };
        assert_eq!(pointer as usize % 4, 0);
        unsafe {
            assert_eq!(pointer.cast::<u16>().read(), 0x1234);
            assert_eq!(pointer.add(2).cast::<u16>().read(), 0x5678);
            assert_eq!(pointer.add(4).cast::<u32>().read(), 0x90abcdef);
            drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                pointer.cast_mut().cast::<u32>(),
                2,
            )));
        }
    }
}
