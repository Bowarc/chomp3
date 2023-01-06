use bitvec::prelude::BitSlice;

static CURSOR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn access(array: &mut BitSlice<u8>, size: usize) -> bitvec::vec::BitVec<u8> {
    let cursor = CURSOR.load(std::sync::atomic::Ordering::Relaxed);
    let o = array[cursor..cursor + size].to_bitvec();
    CURSOR.store(cursor + size, std::sync::atomic::Ordering::Relaxed);
    o
}

// pub fn to_bool_array(vec: bitvec::vec::BitVec<u8>) -> Vec<bool> {
//     let mut o: Vec<bool> = Vec::new();

//     for e in vec.into_vec().iter() {
//         o.push(*e != 0);
//     }

//     o
// }
