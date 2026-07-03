use std::convert::TryInto;
use std::env;
use std::fs;
use std::process;

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn patch_vxp(mut data: Vec<u8>, imsi: &str) -> Vec<u8> {
    let imsi_str = format!("9{}", imsi);
    let imsi_bytes = imsi_str.as_bytes();
    
    let tag_table_offset = read_u32(&data, data.len() - 12) as usize;
    let mut pos = tag_table_offset;

    while pos < data.len() {
        let field_id = read_u32(&data, pos);
        if field_id == 0 {
            break;
        }
        pos += 4;
        
        let mut field_len = read_u32(&data, pos) as usize;
        let len_pos = pos;
        pos += 4;
        
        if field_len == 0 {
            continue;
        }
        
        if field_id == 2 {
            data[pos..pos + 4].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        } else if field_id == 0x12 {
            write_u32(&mut data, len_pos, imsi_bytes.len() as u32);
            data.splice(pos..pos + field_len, imsi_bytes.iter().copied());
            field_len = imsi_bytes.len();
        }
        
        pos += field_len;
    }
    
    data
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.vxp> <output.vxp>", args[0]);
        process::exit(1);
    }
    
    let infile = &args[1];
    let outfile = &args[2];
    
    let imsi = fs::read_to_string("imsi.txt").expect("Failed to read imsi.txt");
    let imsi = imsi.trim();
    
    let data = fs::read(infile).expect("Failed to read input file");
    
    let patched = patch_vxp(data, imsi);
    
    fs::write(outfile, patched).expect("Failed to write output file");
    println!("Patched: {}", outfile);
}
