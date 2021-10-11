use namagiri::posit::Posit;
use namagiri::flquire::FLQuire;

#[test]
fn to_from_test() {
    for i in 0..0b100000000 {
        let a = Posit::<8, 1>(i);
        let q = FLQuire::from(a);
        let aq = Posit::<8, 1>::from(q);
        if !a.is_nar() {
            assert_eq!(a, aq);
        }
    }
}

fn op_test<P, Q>(p: P, q: Q)
    where
        P: Fn(Posit<8, 1>, Posit<8, 1>) -> Posit<8, 1>,
        Q: Fn(FLQuire<8, 1>, FLQuire<8, 1>) -> FLQuire<8, 1>
{
    for i in 0..0b100000000 {
        let a = Posit::<8, 1>(i);
        let qa: FLQuire<8, 1> = a.into();
        for j in 0..0b100000000 {
            let b = Posit::<8, 1>(j);
            let qb: FLQuire<8, 1> = b.into();
            let c = p(a, b);
            let qc:Posit<8, 1> = (q(qa, qb)).into();

            if a.is_nar() || b.is_nar() {
                continue;
            }
            assert_eq!(c, qc);
        }
    }
}

#[test]
fn mul_test() {
    op_test(|x, y| x * y, |x, y| x * y)
}

#[test]
fn add_test() {
    op_test(|x, y| x + y, |x, y| x + y)
}