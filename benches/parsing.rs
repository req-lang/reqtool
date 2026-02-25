use criterion::*;
use reqtool::mock::generator::{self, Generate};

pub mod simple {
    use reqtool::{mock::generator::ExponentialSizeIterator, renderer, syntax, visitor::Walk};

    use super::*;

    pub fn package(c: &mut Criterion) {
        let mut group = c.benchmark_group("parsing::simple::package");
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        for size in ExponentialSizeIterator::new().take(6) {
            let mut generator = generator::Simple::new();
            generator.packages = size;
            let root = generator.generate();
            let mut renderer = renderer::linter::Renderer::new();
            let _ = renderer.walk(&root);
            let input = renderer.source;

            let id = BenchmarkId::from_parameter(size);
            group.throughput(Throughput::Elements(generator.size()));
            group.bench_with_input(id, &input, |b, input| {
                b.iter(|| syntax::NodeParser::default().parse(std::hint::black_box(input)))
            });
        }
        group.finish();
    }

    pub fn depth(c: &mut Criterion) {
        let mut group = c.benchmark_group("parsing::simple::depth");
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        for size in ExponentialSizeIterator::new().take(5) {
            let mut generator = generator::Simple::new();
            generator.depth = size;
            let root = generator.generate();
            let mut renderer = renderer::linter::Renderer::new();
            let _ = renderer.walk(&root);
            let input = renderer.source;

            let id = BenchmarkId::from_parameter(size);
            group.throughput(Throughput::Elements(generator.size()));
            group.bench_with_input(id, &input, |b, input| {
                b.iter(|| syntax::NodeParser::default().parse(std::hint::black_box(input)))
            });
        }
        group.finish();
    }

    pub fn words(c: &mut Criterion) {
        let mut group = c.benchmark_group("parsing::simple::words");
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        for size in ExponentialSizeIterator::new().take(7) {
            let mut generator = generator::Simple::new();
            generator.words = size;
            let root = generator.generate();
            let mut renderer = renderer::linter::Renderer::new();
            let _ = renderer.walk(&root);
            let input = renderer.source;

            let id = BenchmarkId::from_parameter(size);
            group.throughput(Throughput::BytesDecimal(input.len() as u64));
            group.bench_with_input(id, &input, |b, input| {
                b.iter(|| syntax::NodeParser::default().parse(std::hint::black_box(input)))
            });
        }
        group.finish();
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = simple::package, simple::depth, simple::words
);
criterion_main!(benches);
