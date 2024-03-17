use easy_fs::{BlockDevice, EasyFileSystem};

use std::{
    borrow::BorrowMut,
    fs::{read_dir, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

const BLOCK_SZ: usize = 512;
struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block");
    }
}

use clap::{Arg,App};

fn easy_fs_pack() -> std::io::Result<()>{
    let matchs = App::new("EasyFileSystem packer")
        .arg(Arg::with_name("source")
            .short("s")
            .long("source")
            .takes_value(true)
            .help("Executable source dir(with backslash)")
        )
        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .takes_value(true)
            .help("Executable target dir(with backslash)")
        )
        .get_matches();
    let src_path = matchs.value_of("source").unwrap();
    let target_path = matchs.value_of("target").unwrap();
    println!("src_path = {}\ntarget_path={}",src_path,target_path);
    let block_file = Arc::new(BlockFile(Mutex::new(
        {let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}{}",target_path,"fs.img"))?;
        f.set_len(8192*512).unwrap();
        f
        }
    )));
    let efs = EasyFileSystem::create(block_file.clone(), 8192, 1);
    let root_inode = Arc::new(EasyFileSystem::root_inode(&efs));
    let apps: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry|{
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        }).collect();
    for app in apps{
        let mut host_file = File::open(format!("{}{}",target_path,app)).unwrap();
        let mut all_data:Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();
        let inode = root_inode.create(app.as_str()).unwrap();
        inode.write_at(0, all_data.as_slice());
    }
    for app in root_inode.ls(){
        println!("{}",app);
    }
    Ok(())
}

fn main() {
    easy_fs_pack().expect("Error when packing easy-fs!");
}

#[test]
fn efs_test() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?;
        f.set_len(8192 * 512).unwrap();
        f
    })));
    let efs = EasyFileSystem::create(block_file.clone(), 4096, 1);
    //let _efs1 = EasyFileSystem::open(block_file.clone());
    let root_node = EasyFileSystem::root_inode(&efs);
    root_node.create("filea");
    root_node.create("fileb");
    let (bl_id, bl_of) = root_node.stat();
    println!(
        "root_node's block_id is {},block_offset is {}",
        bl_id, bl_of
    );
    for name in root_node.ls() {
        println!("{}", name);
    }
    let filea = root_node.find("filea").unwrap();
    let (bl_id, bl_of) = filea.stat();
    println!("fila's block_id is {},block_offset is {}", bl_id, bl_of);
    let greet_str = "Hello, world!";
    filea.write_at(0, greet_str.as_bytes());
    let mut buffer = [0u8; 233];
    let len = filea.read_at(0, &mut buffer);
    assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap(),);

    Ok(())
}
