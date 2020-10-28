     #[allow(unused_braces)]
     _crit.bench_function(r###"`/*EXPRESSION*/`"###, |_bencher| _bencher.iter(|| black_box({ /*EXPRESSION*/ })));
