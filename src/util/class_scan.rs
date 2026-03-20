use crate::classfile_constants::{JVM_CONSTANT_Class, JVM_CONSTANT_Double, JVM_CONSTANT_Dynamic, JVM_CONSTANT_Fieldref, JVM_CONSTANT_Float, JVM_CONSTANT_Integer, JVM_CONSTANT_InterfaceMethodref, JVM_CONSTANT_InvokeDynamic, JVM_CONSTANT_Long, JVM_CONSTANT_MethodHandle, JVM_CONSTANT_MethodType, JVM_CONSTANT_Methodref, JVM_CONSTANT_Module, JVM_CONSTANT_NameAndType, JVM_CONSTANT_Package, JVM_CONSTANT_String, JVM_CONSTANT_Utf8};
use crate::common::error::{MessageError, Result};

#[repr(C, align(8))]
#[derive(Debug)]
pub struct DataRange {
    pub start: usize,
    pub end: usize,
}

#[repr(C, align(8))]
#[derive(Debug)]
pub struct MethodRange {
    pub start: usize,
    pub end: usize,
    pub code_start: usize,
    pub code_end: usize,
}

/// consts 中每个元素为每个常量的截止索引（不含索引指向的值），首元素为第一个常量的开始索引
#[repr(C, align(8))]
#[derive(Debug)]
pub struct SimpleClassInfo {
    pub consts: Vec<usize>,
    pub fields_start: usize,
    pub methods_start: usize,
    pub method_items: Vec<MethodRange>,
    pub attributes_start: usize,
}

const CODE_ATTR_NAME: &[u8] = "Code".as_bytes();

const CODE_ATTR_NAME_LEN: usize = CODE_ATTR_NAME.len();

#[inline]
pub fn fast_scan_class(data: & [u8]) -> Result<SimpleClassInfo> {
    // magic + minor_version + major_version
    let mut index = 8;
    let constant_size = get_u16_from_data(data, &mut index)?;
    let constant_size = constant_size as usize;
    let mut data_key_index = 0;
    let mut consts = Vec::with_capacity(constant_size);
    unsafe {
        consts.set_len(constant_size);
    }
    consts[0] = index;
    let mut find_code = true;

    let mut code_index = 0;
    let mut i = 1;
    while i < constant_size {
        match get_constant_value_size(data, &mut index, find_code)? {
            1 => {
                find_code = false;
                code_index = i;
            }
            2 => {
                consts[i] = index;
                i += 1;
            }
            _ => {}
        }
        consts[i] = index;
        i += 1;
    }

    let constants_end = index;
    // access_flags + class_index + superclass_index
    index += 6;
    // interface
    let interface_size = get_u16_from_data(data, &mut index)?;
    index += (interface_size as usize) << 1;
    // field
    let fields_start = index;
    handle_field_or_method(data, &mut index)?;
    // method
    let methods_start = index;
    // handle_field_or_method(data, &mut index)?;
    let code_index_bytes = (code_index as u16).to_be_bytes();
    let size = get_u16_from_data(data, &mut index)?;
    let size = size as usize;
    let mut method_codes = Vec::with_capacity(size);
    unsafe {
        method_codes.set_len(size);
    }
    for i in 0..size {
        let method_start = index;
        // access_flags + name + descriptor
        index += 6;
        let attr_size = get_u16_from_data(data, &mut index)?;
        let mut code_range = (0, 0);
        for _ in 0..attr_size {
            // name
            let start = index;
            index += 2;
            let data_size = get_u32_from_data(data, &mut index)?;
            index += data_size as usize;
            if &data[start..start+2] == &code_index_bytes {
                code_range = (start, index);
            }
        }
        method_codes[i] = MethodRange {
            start: methods_start,
            end: index,
            code_start: code_range.0,
            code_end: code_range.1,
        };
    }


    // attribute
    let attributes_start = index;
    Ok(SimpleClassInfo {
        consts,
        fields_start,
        methods_start,
        method_items: method_codes,
        attributes_start,
    })
}

#[inline(always)]
fn handle_attributes(data: &[u8], index: &mut usize) -> Result<()> {
    let attr_size = get_u16_from_data(data, index)?;
    for _ in 0..attr_size {
        // name
        *index += 2;
        let data_size = get_u32_from_data(data, index)?;
        *index += data_size as usize;
    }
    Ok(())
}

#[inline(always)]
pub fn handle_field_or_method(data: &[u8], index: &mut usize) -> Result<()> {
    let size = get_u16_from_data(data, index)?;
    for _ in 0..size {
        // access_flags + name + descriptor
        *index += 6;
        handle_attributes(data, index)?;
    }
    Ok(())
}

#[inline(always)]
fn get_constant_value_size(data: &[u8], index: &mut usize, find_code: bool) -> Result<u8> {
        let type_ = match data.get(*index) {
            None => {
                return Err(MessageError::new("读取常量类型时越界"));
            }
            Some(v) => *v
        };
        *index += 1;
        *index += match type_ {
            JVM_CONSTANT_Utf8 => {
                let str_size = get_u16_from_data(data, index)?;
                let str_size = str_size as usize;
                if find_code && str_size == CODE_ATTR_NAME_LEN {
                    let start = *index;
                    *index += CODE_ATTR_NAME_LEN;
                    if *index > data.len() {
                        return Err(MessageError::new("读取utf8越界"))
                    }

                    return Ok((&data[start..*index] == CODE_ATTR_NAME) as u8);
                }
                str_size
            }
            JVM_CONSTANT_Integer | JVM_CONSTANT_Float => {
                size_of::<u32>()
            }
            JVM_CONSTANT_Long | JVM_CONSTANT_Double => {
                // long and double used 2 index
                *index += size_of::<i64>();
                return Ok(2);
            }
            JVM_CONSTANT_Class |
            JVM_CONSTANT_String | JVM_CONSTANT_MethodType |
            JVM_CONSTANT_Module | JVM_CONSTANT_Package => {
                size_of::<u16>()
            }
            JVM_CONSTANT_Fieldref | JVM_CONSTANT_Methodref |
            JVM_CONSTANT_InterfaceMethodref | JVM_CONSTANT_NameAndType |
            JVM_CONSTANT_Dynamic | JVM_CONSTANT_InvokeDynamic => {
                size_of::<[u16; 2]>()
            }
            JVM_CONSTANT_MethodHandle => {
                size_of::<u16>() + size_of::<u8>()
            }
            _ => {
                0
            }
        };
    Ok(0)
}

#[inline(always)]
pub fn get_u16_from_data(data: &[u8], index: &mut usize) -> Result<u16> {
    let start = *index;
    *index += 2;
    if *index > data.len() {
        return Err(MessageError::new("读取u16越界"))
    }
    unsafe {
        let ptr = data.as_ptr().add(start) as *const u16;
        Ok(u16::from_be(ptr.read_unaligned()))
    }
}

#[inline(always)]
pub fn get_u32_from_data(data: &[u8], index: &mut usize) -> Result<u32> {
    let start = *index;
    *index += 4;
    if *index > data.len() {
        return Err(MessageError::new("读取u32越界"))
    }
    unsafe {
        let ptr = data.as_ptr().add(start) as *const u32;
        Ok(u32::from_be(ptr.read_unaligned()))
    }
}