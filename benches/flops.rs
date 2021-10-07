use criterion::{black_box as bb, criterion_group, criterion_main, Criterion};

use namagiri::posit::Posit;

fn criterion_p6e1(c: &mut Criterion) {
    const X: Posit<8, 3> =  Posit::<8, 3>(0b00010010);
    const Y: Posit<8, 3> =  Posit::<8, 3>(0b01100011);

    const XF: f32 = 3.05176e-05;
    const YF: f32 = 768.0;

    c.bench_function("add_p", |b| b.iter(|| bb(X) + bb(Y)));
    c.bench_function("mul_p", |b| b.iter(|| bb(X) * bb(Y)));
    c.bench_function("add_f", |b| b.iter(|| bb(XF) + bb(YF)));
    c.bench_function("mul_f", |b| b.iter(|| bb(XF) * bb(YF)));

//    c.bench_function("sub", |b| b.iter(|| bb(X) - bb(Y)));
//    c.bench_function("div", |b| b.iter(|| bb(X) / bb(Y)));
}

criterion_group!(benches, criterion_p6e1);
criterion_main!(benches);