use std::{fs::File, io::{Read, Write}, process::exit};

use crate::huffman::*;

mod huffman;

fn main() {
    println!("【哈夫曼压缩】");
    loop {
        println!("1. 压缩文件\t2. 解压文件\t3. 退出");
        match read().as_str() {
            "1" => {
                println!("待压缩文件路径名：");
                let file = read();
                hfm_compress(&file);
                break;
            },
            "2" => {
                println!("待解压文件路径名：");
                let file = read();
                let config_file = format!("{}.config", file);

                println!("请输入保存文件路径：");
                let save_file = read();
                hfm_decompress(&file, &config_file, &save_file);
                break;
            },
            "3" => exit(1),
            _ => ()
        }
    }
}

/// 读取终端输入
fn read() -> String {
    let mut cmd = String::new();
    std::io::stdin()
        .read_line(&mut cmd)
        .expect("failed to read");
    cmd.trim().to_string()
}

fn hfm_compress(file: &str) {
    let filename = {
        let mut arr = file.split(".").collect::<Vec<&str>>();
        arr.pop();
        arr.join(".")
    };
    // 打开文件并读取字符串到内存中
    let mut input_file = File::open(file).expect("未找到该文件");
    let mut source_buf = String::new();
    input_file.read_to_string(&mut source_buf).expect("读取文件失败");

    // 哈夫曼编码
    let (u8_arr, config) = HuffmanCodec::encode(&source_buf);

    // 创建压缩文件
    let output_file_name = format!("{}.hfm", filename);
    let mut output_file = File::create(&output_file_name).unwrap();
    output_file.write(&u8_arr).unwrap();

    // 创建压缩配置文件
    let output_cfg_file_name = format!("{}.hfm.config", filename);
    let mut output_cfg_file = File::create(&output_cfg_file_name).unwrap();
    output_cfg_file.write(config.as_bytes()).unwrap();
    println!("\n压缩成功！\n文件保存为: {}\n配置文件: {}", output_file_name, output_cfg_file_name);
    let size_before = source_buf.as_bytes().len();
    let size_after = u8_arr.len();
    println!("压缩前大小：{} 字节", size_before);
    println!("压缩后大小：{} 字节", size_after);
    println!("压缩比率： {:.2}%", ((size_after as f64 / size_before as f64) * 100.0));
}

fn hfm_decompress(file: &str, config_file: &str, save_file: &str) {
    // 压缩文件
    let mut encodede_file = File::open(file).expect(format!("未找到文件:{}",file).as_str());
    let mut buf = vec![];
    encodede_file.read_to_end(&mut buf).unwrap();

    // 读取配置文件
    let mut config = File::open(config_file).expect(format!("未找到配置文件:{}", config_file).as_str());
    let mut buf2 = String::new();
    config.read_to_string(&mut buf2).unwrap();
    
    // 构建配置
    let char_map = DecodeConfig::build(&buf2);

    // 解码
    let result = HuffmanCodec::decode(&buf, &char_map);

    let mut savef = File::create(save_file).unwrap();
    savef.write(result.as_bytes()).unwrap();

    println!("\n解压成功！\n文件已保存至：{}", save_file);
}
