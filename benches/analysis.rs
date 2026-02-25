use criterion::*;
use reqtool::{
    mock::generator::{self, Generate},
    Analysis,
};

pub mod simple {
    use reqtool::mock::generator::ExponentialSizeIterator;

    use super::*;

    pub fn package(c: &mut Criterion) {
        let mut group = c.benchmark_group("analysis::simple::package");
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        for size in ExponentialSizeIterator::new().take(7) {
            let mut generator = generator::Simple::new();
            generator.packages = size;

            let root = generator.generate();
            let id = BenchmarkId::from_parameter(size);
            group.throughput(Throughput::Elements(generator.size()));
            group.bench_with_input(id, &root, |b, root| {
                b.iter(|| Analysis::from(std::hint::black_box(root)));
            });
        }
        group.finish();
    }

    pub fn depth(c: &mut Criterion) {
        let mut group = c.benchmark_group("analysis::simple::depth");
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        for size in ExponentialSizeIterator::new().take(5) {
            let mut generator = generator::Simple::new();
            generator.depth = size;

            let root = generator.generate();
            let id = BenchmarkId::from_parameter(size);
            group.throughput(Throughput::Elements(generator.size()));
            group.bench_with_input(id, &root, |b, root| {
                b.iter(|| Analysis::from(std::hint::black_box(root)));
            });
        }
        group.finish();
    }
}

criterion_group!(benches, simple::package, simple::depth);
criterion_main!(benches);
