
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// THIS EXAMPLE ~ IS REAL(EXCEPT FOR AMOUNT OF FIELDS) 
// USE CASE THAT LED TO WRITING OF THIS CODE
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct Object {
    some_values: Vec<u32>,
}

#[ref_struct::ref_struct(
    use_cow,
    name(Cowed),
    ignore(named_object_ignore),
    clone(some_inner_info_clone),
    derive(Debug, serde::Serialize, serde::Deserialize),
    ignore_struct(name(Ignore), derive(Debug))
)]
#[derive(Debug, PartialEq, Eq)]
struct SplitedSerDeser {
    named_object_ignore: HashMap<String, Object>,
    some_inner_info_ref: Vec<u8>,
    some_inner_info_clone: u32,
}


fn test_helper_init() -> SplitedSerDeser {
    let mut named_object_ignore = HashMap::new();
    named_object_ignore.insert("power_of_2".into(), Object { some_values: vec![2, 4, 32, 128] });
    named_object_ignore.insert("random".into(), Object { some_values: vec![12, 24, 48] });
    named_object_ignore.insert("11111".into(), Object { some_values: vec![1, 1, 1, 1, 1] });
    named_object_ignore.insert("dead".into(), Object { some_values: vec![0xD, 0xE, 0xA, 0xD] });

    SplitedSerDeser {
        named_object_ignore,
        some_inner_info_ref: vec![2, 3, 4, 6, 7],
        some_inner_info_clone: 0xA115E,
    }
}

#[test]
pub fn test_ser_deser() {
    let splited = test_helper_init();
    let cowed = Cowed::new(&splited);

    // for example we can write this bytes in file and then read them
    // lets pretend that it is the case
    let mut bytes = Vec::<u8>::new();
    let mut serializer = rmp_serde::Serializer::new(&mut bytes);
    cowed.serialize(&mut serializer).unwrap();
    
    // the point is the same as the last time: 
    // lets pretend that this map of bytes is
    //      diff named files of bytes that we write and then read 
    let mut object_bytes = HashMap::new();
    for (name, obj) in &splited.named_object_ignore {
        let mut bytes = Vec::<u8>::new();
        let mut serializer = rmp_serde::Serializer::new(&mut bytes);
        obj.serialize(&mut serializer).unwrap();
        object_bytes.insert(name.to_owned(), bytes);
    }

    drop(cowed);
    drop(splited);

    // [+] pretend reading files
    let named_object_ignore = {
        let mut named_object_ignore = HashMap::new();
        for (name, bytes) in object_bytes {    
            let mut deserializer = rmp_serde::Deserializer::new(bytes.as_slice());
            let obj: Object = serde::Deserialize::deserialize(&mut deserializer).unwrap();
            named_object_ignore.insert(name, obj);
        }
        named_object_ignore
    };
    let ignore = Ignore {
        named_object_ignore,
    };

    let owned = {
        let mut deserializer = rmp_serde::Deserializer::new(bytes.as_slice());
        let cowed: Cowed = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        cowed
    };
    // [-] pretend reading files

    let loaded_value = owned.merge(ignore);
    let initial_value = test_helper_init();

    assert_eq!(loaded_value, initial_value);
}
