use criterion::{black_box, criterion_group, criterion_main, Criterion};

@INCLUDES@

#[allow(redundant_semicolons)]
fn timeit(_crit: &mut Criterion) {
    @SETUP@;
    @EXPRESSIONS@
}

criterion_group!(benches, timeit);
criterion_main!(benches);
