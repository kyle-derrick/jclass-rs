use jclass::attribute_info::CodeAttribute;
use jclass::common::constants::CODE_TAG;
use jclass::constant_pool::ConstantValue;
use jclass::jclass_info::JClassInfo;
use jclass::util::class_scan::fast_scan_class;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fs::{read, File};
use std::io::{BufWriter, Cursor, Read};
use std::time::Instant;

#[cfg(target_os = "linux")]
const FILE_PATH: &str = "/mnt/d/data/code/git-cmp/fta-vpn/fta-vpn-module/build/classes/java/main/com/ws/ftavpn/plugin/firewall/service/FirewallPluginService.class";
#[cfg(target_os = "windows")]
const FILE_PATH: &str = "D:\\data\\code\\git-cmp\\fta-vpn\\fta-vpn-module\\build\\classes\\java\\main\\com\\ws\\ftavpn\\plugin\\firewall\\service\\FirewallPluginService.class";

#[test]
fn base_test() {
    let content = File::open(FILE_PATH).unwrap();
    let now = Instant::now();
    let info = JClassInfo::from_reader(&mut content.into());
    println!(">> {:?}", now.elapsed().as_nanos());
    if let Ok(_) = info {
        // println!("{:?}", &info);
    }

    let content = read(FILE_PATH).unwrap();
    let mut t = 0;
    let mut min_t = u128::MAX;
    let mut max_t = 0;
    let mut avg_t = 0;
    for _ in 0..10000 {
        // let content_ref = content.clone();
        // let cursor = Cursor::new(content_ref);
        let cursor = Cursor::new(&content);
        let now = Instant::now();
        let info = JClassInfo::from_reader(&mut cursor.into());
        if let Ok(info) = info {
            let constant_count = info.constant_pool.get_constant_count();
            let mut index_set = HashSet::with_capacity(5);
            for i in 0..constant_count {
                let value = info.constant_pool.get_constant_item(i);
                match value {
                    ConstantValue::ConstantString(utf8_index) => {
                        if let ConstantValue::ConstantUtf8(utf8_str) = info.constant_pool.get_constant_item(*utf8_index) {
                            if utf8_str == CODE_TAG {
                                index_set.insert(i);
                            }
                        }
                    }
                    ConstantValue::ConstantUtf8(utf8_str) => {
                        if utf8_str == CODE_TAG {
                            index_set.insert(i);
                        }
                    }
                    _ => {}
                }
            }
            for method_info in info.methods {
                let mut has_code = false;
                for attribute_info in method_info.attributes {
                    if index_set.contains(&attribute_info.name) {
                        if let Ok(attr) = CodeAttribute::new_with_data(&attribute_info.data) {
                            if attr.codes.len() <= 0 {
                                println!("{}", attr.codes.len());
                            }
                            has_code = true;
                        }
                    }
                }
                if !has_code && method_info.name != 161 {
                    println!("not found code");
                }
            }
        }
        let duration = now.elapsed();
        let n = duration.as_nanos();
        t += n;
        min_t = min(n, min_t);
        max_t = max(n, max_t);
        avg_t += n;

    }
    println!(">> {:?}", t);
    println!(">> min: {:?}", min_t);
    println!(">> max: {:?}", max_t);
    println!(">> avg: {:?}", avg_t/10000);
}

#[test]
fn test_parser() {
    let content = File::open(FILE_PATH).unwrap();
    let now = Instant::now();
    let  info = JClassInfo::from_reader(&mut content.into());
    println!(">> {:?}", now.elapsed().as_nanos());
    if let Ok(info) = info {
        println!("{:?}", &info);
    }

    let content = read(FILE_PATH).unwrap();
    let mut t = 0;
    for _ in 0..10000 {
        // let content_ref = content.clone();
        // let cursor = Cursor::new(content_ref);
        let cursor = Cursor::new(&content);
        let now = Instant::now();
        let mut _info = JClassInfo::from_reader(&mut cursor.into());
        let duration = now.elapsed();
        t += duration.as_nanos();
    }
    println!(">> {:?}", t);
}

#[test]
fn test_to_bytes() {
    let mut content = File::open(FILE_PATH).unwrap();
    let mut content_data = Vec::new();
    content.read_to_end(&mut content_data).unwrap();
    println!("origin size: {}", content_data.len());
    let cursor = Cursor::new(&content_data);
    let info = JClassInfo::from_reader(&mut cursor.into()).unwrap();
    let size = info.byte_size();
    let now = Instant::now();
    let mut arr = Vec::new();
    {
        let mut writer = BufWriter::new(&mut arr).into();
        info.write_to(&mut writer).unwrap();

        let use_time = now.elapsed();
        println!(": {}", use_time.as_nanos());
    }
    if content_data != arr {
        println!("\n\nto byte not equals with origin data")
    }
    println!("{size}");
    println!("{}", arr.len());
    let now = Instant::now();
    let size = info.byte_size();
    let mut arr = Vec::with_capacity(size);
    {
        let mut writer = BufWriter::new(&mut arr).into();
        info.write_to(&mut writer).unwrap();

        let use_time = now.elapsed();
        println!(": {}", use_time.as_nanos());
    }
    if content_data != arr {
        println!("\n\nto byte not equals with origin data: \n{:?}\n{:?}", &content_data, &arr);
    }
    println!("{size}");
    println!("{}", arr.len());


    let constant_count = info.constant_pool.get_constant_count();
    let mut index_set = HashSet::with_capacity(5);
    for i in 0..constant_count {
        let value = info.constant_pool.get_constant_item(i);
        match value {
            ConstantValue::ConstantString(utf8_index) => {
                if let ConstantValue::ConstantUtf8(utf8_str) = info.constant_pool.get_constant_item(*utf8_index) {
                    if utf8_str == CODE_TAG {
                        index_set.insert(i);
                    }
                }
            }
            ConstantValue::ConstantUtf8(utf8_str) => {
                if utf8_str == CODE_TAG {
                    index_set.insert(i);
                }
            }
            _ => {}
        }
    }
    for method_info in info.methods {
        let mut has_code = false;
        for attribute_info in method_info.attributes {
            if index_set.contains(&attribute_info.name) {
                if let Ok(attr) = CodeAttribute::new_with_data(&attribute_info.data) {
                    if attr.codes.len() <= 0 {
                        println!("{}", attr.codes.len());
                    }
                    has_code = true;
                    let bytes = attr.to_bytes().unwrap();
                    if bytes != attribute_info.data {
                        println!("\n\ncode to byte not equals with origin data:\n{:?}\n---\n{:?}", &attribute_info.data, &bytes)
                    }
                }
            }
        }
        if !has_code && method_info.name != 161 {
            println!("not found code");
        }
    }
}

#[test]
fn test_class_check() {
    let mut content = File::open(FILE_PATH).unwrap();
    let mut content_data = Vec::new();
    content.read_to_end(&mut content_data).unwrap();
    println!("class info: {:?}", fast_scan_class(&content_data));
    let now = Instant::now();
    for _ in 0..10000 {
        let _ = fast_scan_class(&content_data);
    }
    println!(": {}", now.elapsed().as_nanos());
    println!("class info: {:?}", fast_scan_class(&content_data));
    let now = Instant::now();
    for _ in 0..10000 {
        let _ = fast_scan_class(&content_data);
    }
    println!(": {}", now.elapsed().as_nanos());
}

#[test]
fn test_url_class_parse() {
    let mut content = File::open("URL.class").unwrap();
    let mut content_data = Vec::new();
    content.read_to_end(&mut content_data).unwrap();
    let info = JClassInfo::from_reader(&mut Cursor::new(content_data).into());
    println!("{:?}", &info);
}