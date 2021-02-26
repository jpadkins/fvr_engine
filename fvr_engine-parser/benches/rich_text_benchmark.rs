use criterion::*;

use fvr_engine_parser::prelude::*;

const TEST_RICH_STR: &str = "<l:t><o:f>\n<fc:Y><bc:k><<<oc:k>Hello, <l:c><o:t><fc:k><oc:R>world<l:t>\n<o:f><fc:Y>!";

pub fn benchmark(c: &mut Criterion) {
  let short: &str = TEST_RICH_STR;
  let long: &str = &TEST_RICH_STR.to_string().repeat(1024);

  // Benchmark the parser on a short rich string.
  c.bench_with_input(BenchmarkId::new("rich_text_parser", "a short string"), &short, |b, &short| {
    b.iter(|| {
      let _parsed_rich_text = parse_rich_text(short.into()).unwrap();
    })
  });

  // Benchmark the parser on a long rich string.
  c.bench_with_input(BenchmarkId::new("rich_text_parser", "a long string"), &long, |b, &long| {
    b.iter(|| {
      let _parsed_rich_text = parse_rich_text(long.into()).unwrap();
    })
  });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);