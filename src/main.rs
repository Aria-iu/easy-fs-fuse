use easy_fs::{BlockDevice, EasyFileSystem};

use std::{
    borrow::BorrowMut,
    fs::{File, OpenOptions},
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

fn main() {
    println!("Hello, world!");
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
