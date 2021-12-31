#[macro_use]
macro_rules! read {
    ($fd:expr, $buflen:expr) => {
        {
            let mut bufid = [0_u8; $buflen];
            sys!(read($fd, bufid.as_mut_ptr() as *mut _, $buflen));
            bufid
        }
    }
}

macro_rules! uuid {
    ($fd:expr) => {
        {
            let d1 = u32::from_le_bytes(read!($fd, 4));
            let d2 = u16::from_le_bytes(read!($fd, 2));
            let d3 = u16::from_le_bytes(read!($fd, 2));
            let d4: [u8; 8] = read!($fd, 8);
            format!(
                "{:08x?}-{:04x?}-{:04x?}-{:02x?}{:02x?}-{:02x?}{:02x?}{:02x?}{:02x?}{:02x?}{:02x?}",
                d1, d2, d3,
                d4[0], d4[1], d4[2], d4[3], d4[4], d4[5], d4[6], d4[7])
        }
    }
}

fn main() {
    const LB512: u64 = 512 /* LBS */;

    let fd = sys!(open(c_str!("/dev/sda"), libc::O_RDONLY));
    sys!(lseek(fd, LB512 as i64, libc::SEEK_SET));

    println!("signature:    {:?}", std::str::from_utf8(&read!(fd, 8)));
    println!("revision:     {}", u32::from_le_bytes(read!(fd, 4)));
    println!("header size:  {}", u32::from_le_bytes(read!(fd, 4)));
    println!("header CRC32: {:x?}", u32::from_le_bytes(read!(fd, 4)));
    println!("reserved:     {:?}", read!(fd, 4));
    println!("current LBA:     {}", u64::from_le_bytes(read!(fd, 8)));
    println!("backup LBA:  {}", u64::from_le_bytes(read!(fd, 8)));
    println!("first usable LBA for partitions: {}", u64::from_le_bytes(read!(fd, 8)));
    println!("last usable LBA: {}", u64::from_le_bytes(read!(fd, 8)));
    println!("GUID: {:x?}", read!(fd, 16));

    let part_start = u64::from_le_bytes(read!(fd, 8));
    println!("starting LBA of array of partition entries: {}", part_start);

    let num_parts = u32::from_le_bytes(read!(fd, 4));
    println!("number of partition entries in array: {}", num_parts);
    println!("size of a single partition entry: {}", u32::from_le_bytes(read!(fd, 4)));
    println!("CRC32 of partition entries array in little endian: {:x?}", u32::from_le_bytes(read!(fd, 4)));


    let pstart = part_start.mul(LB512);
    sys!(lseek(fd, pstart as i64, libc::SEEK_SET));

    for _ in 0..num_parts {
        let uuid = uuid!(fd);
        let partuuid = uuid!(fd);
        let first_lba = u64::from_le_bytes(read!(fd, 8));
        let last_lba = u64::from_le_bytes(read!(fd, 8));
        let attributes = u64::from_le_bytes(read!(fd, 8));
        let name = read!(fd, 128-16-16-8-8-8);

        if uuid != "00000000-0000-0000-0000-000000000000" {
            println!("PARTTYPE: {:?}", uuid);
            println!("PARTUUID: {:?}", partuuid);
            println!("FIRST_LBA: {:?}", first_lba);
            println!("LAST_LBA: {:?}", last_lba);
            println!("ATTRIBUTES: {:?}", attributes);
            println!("name: {:?}", name);
        }
    }

    sys!(close(fd));
}
