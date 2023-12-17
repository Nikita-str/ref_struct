

#[ref_struct::ref_struct(ignore(ignore_a, ignore_b), clone(clone_a))]
struct SimpleStruct {
    ignore_a: u32,
    ref_a: Vec<u8>,
    ref_b: Vec<u32>,
    copy_a: u64,
    ignore_b: Vec<Vec<u16>>,
}

