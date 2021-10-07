use namagiri::posit::Posit;

fn main() {
    for i in 0..0b1000000 {
        let p = Posit::<6, 1>(i);
        for j in 0..0b1000000 {
            let o = Posit::<6, 1>(j);
            println!("{}", f32::from(p + o));
        }
    }
}
