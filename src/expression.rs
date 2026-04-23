     #[allow(unused_braces)]
     _crit.bench_function(r###"`/*EXPRESSION*/`"###, |_bencher| _bencher.iter(||
	     std::hint::black_box({ /*EXPRESSION*/ })
     ));
