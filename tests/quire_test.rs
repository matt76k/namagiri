use namagiri::posit::Posit;
use namagiri::quire::Quire;

#[test]
fn to_from_test() {
    for i in 0..0b100000000 {
        let a = Posit::<8, 1>(i);
        let q = Quire::from(a);
        let aq = Posit::<8, 1>::from(q);
        if !a.is_nar() {
            assert_eq!(a, aq);
        }
    }
}