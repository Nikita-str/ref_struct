
#[ref_struct::ref_struct(ignore(ignore_a, ignore_b), derive(Clone, Debug, PartialEq, Eq), name(TwoVecAndCopy), clone(copy_a))]
struct SimpleStruct {
    ignore_a: u32,
    ref_a: Vec<u8>,
    ref_b: Vec<u32>,
    copy_a: u64,
    ignore_b: Vec<Vec<u16>>,
}

#[test]
pub fn test_01() {
    let a = vec![12, 23, 34];
    let b0 = vec![27, 29, 31];
    let b1 = vec![72, 92, 13];
    let clone = 0xC10_E;

    let x = SimpleStruct {
        ignore_a: 42,
        ref_a: a.clone(),
        ref_b: b0.clone(),
        copy_a: clone,
        ignore_b: vec![vec![2, 4], vec![32, 8, 16]],
    };
    let x_link = TwoVecAndCopy::new(&x);

    let y = SimpleStruct {
        ignore_a: 1421,
        ref_a: a.clone(),
        ref_b: b0.clone(),
        copy_a: clone,
        ignore_b: vec![vec![2, 2, 2, 2], vec![8, 8, 8]],
    };
    let y_link = TwoVecAndCopy::new(&y);

    assert_ne!(&x.ignore_a, &y.ignore_a);
    assert_ne!(&x.ignore_b, &y.ignore_b);

    assert_eq!(x_link.copy_a, clone);
    assert_eq!(x_link.ref_a, &a);
    assert_eq!(x_link.ref_b, &b0);

    assert_eq!(x_link, y_link);

    let ne = SimpleStruct {
        ignore_a: 1421,
        ref_a: a.clone(),
        ref_b: b1.clone(),
        copy_a: clone,
        ignore_b: vec![vec![2, 2, 2, 2], vec![8, 8, 8]],
    };
    let ne_link = TwoVecAndCopy::new(&ne);
    
    assert_ne!(x_link, ne_link);
    assert_ne!(y_link, ne_link);
}
