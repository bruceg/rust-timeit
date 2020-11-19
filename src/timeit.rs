#![allow(unused_imports)]
#![allow(redundant_semicolons)]

use criterion::{
    black_box, criterion_group, criterion_main,
    measurement::{Measurement, WallTime},
    Criterion,
};

/*USES*/

/*INCLUDES*/

fn timeit<T: 'static + Measurement>(_crit: &mut Criterion<T>) {
    /*SETUP*/

    /*EXPRESSIONS*/
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(/*TIMER*/);
    targets = timeit
);
criterion_main!(benches);
