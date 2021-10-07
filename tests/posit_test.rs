use namagiri::posit::Posit;

#[test]
fn to_from_test() {
    for i in 0..0b100000000 {
        let a = Posit::<8, 1>(i);
        let f: f32 = f32::from(a);
        let af = Posit::<8, 1>::from(f);
        assert_eq!(a, af);
    }
}
