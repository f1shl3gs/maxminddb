use std::net::IpAddr;
use std::str::FromStr;

use criterion::{criterion_group, criterion_main, Criterion};
use fake::faker::internet::raw::IPv4;
use fake::locales::EN;
use fake::Fake;

// Generate `count` IPv4 addresses
#[must_use]
pub fn generate_ipv4(count: u64) -> Vec<IpAddr> {
    let mut ips = Vec::new();
    for _i in 0..count {
        let val: String = IPv4(EN).fake();
        let ip: IpAddr = FromStr::from_str(&val).unwrap();
        ips.push(ip);
    }
    ips
}

fn bench(c: &mut Criterion) {
    let ips = generate_ipv4(100);
    let path = "GeoLite2-City.mmdb";

    let mut group = c.benchmark_group("bench");

    group.bench_function("in-memory", |b| {
        let reader = maxminddb::Reader::open_file(path).unwrap();

        b.iter(|| {
            for ip in ips.iter() {
                let _ = reader.lookup::<maxminddb::City>(*ip);
            }
        })
    });

    group.bench_function("mmap", |b| {
        let reader = maxminddb::Reader::open_mmap(path).unwrap();

        b.iter(|| {
            for ip in ips.iter() {
                let _ = reader.lookup::<maxminddb::City>(*ip);
            }
        })
    });

    group.bench_function("geoip2", |b| {
        let data = std::fs::read(path).unwrap();
        let reader = ::geoip2::Reader::<::geoip2::City>::from_bytes(&data).unwrap();

        b.iter(|| {
            for ip in ips.iter() {
                let _ = reader.lookup(*ip);
            }
        })
    });

    group.finish()
}

criterion_group!(benches, bench);
criterion_main!(benches);
