use criterion::{black_box, criterion_group, criterion_main, Criterion};
use freqshow::{Complex, FreqImage};

fn make_test_image(size: u32) -> FreqImage {
    let data: Vec<Complex<f64>> = (0..(size * size) as usize)
        .map(|i| Complex::new(i as f64 / (size * size) as f64, 0.0))
        .collect();
    FreqImage {
        width: size,
        height: size,
        data,
    }
}

fn bench_fft_forward(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_forward");
    for size in [64, 256, 512] {
        group.bench_function(format!("{size}x{size}"), |b| {
            b.iter_batched(
                || make_test_image(size),
                |mut fi| {
                    fi.fft_forward();
                    black_box(fi)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_fft_inverse(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_inverse");
    for size in [64, 256, 512] {
        group.bench_function(format!("{size}x{size}"), |b| {
            b.iter_batched(
                || {
                    let mut fi = make_test_image(size);
                    fi.fft_forward();
                    fi
                },
                |mut fi| {
                    fi.fft_inverse();
                    black_box(fi)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_fft_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_roundtrip");
    for size in [64, 256, 512] {
        group.bench_function(format!("{size}x{size}"), |b| {
            b.iter_batched(
                || make_test_image(size),
                |mut fi| {
                    fi.fft_forward();
                    fi.fft_inverse();
                    black_box(fi)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_fftshift(c: &mut Criterion) {
    let mut group = c.benchmark_group("fftshift");
    for size in [64, 256, 512] {
        group.bench_function(format!("{size}x{size}"), |b| {
            let fi = make_test_image(size);
            b.iter(|| black_box(fi.fftshift()));
        });
    }
    group.finish();
}

fn bench_low_pass_mask(c: &mut Criterion) {
    let mut group = c.benchmark_group("low_pass_mask");
    for size in [64, 256, 512] {
        group.bench_function(format!("{size}x{size}"), |b| {
            let fi = make_test_image(size);
            b.iter(|| black_box(fi.low_pass_mask(0.10, 0.02)));
        });
    }
    group.finish();
}

fn bench_apply_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_filter");
    for size in [64, 256, 512] {
        group.bench_function(format!("{size}x{size}"), |b| {
            let mask = vec![0.5f64; (size * size) as usize];
            b.iter_batched(
                || make_test_image(size),
                |mut fi| {
                    fi.apply_filter(&mask);
                    black_box(fi)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_fft_forward,
    bench_fft_inverse,
    bench_fft_roundtrip,
    bench_fftshift,
    bench_low_pass_mask,
    bench_apply_filter,
);
criterion_main!(benches);
