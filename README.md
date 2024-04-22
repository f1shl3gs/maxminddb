# MaxMind DB

This library reads the MaxMind DB format, including the GeoIP2 and GeoLite2 databases.

## Features
- lightweight
- less dependencies

## Mmap
Mmap will use less memory than in-memory implementation.

## Bench
```text
bench/in-memory         time:   [119.01 µs 120.92 µs 123.78 µs]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
bench/mmap              time:   [125.66 µs 129.31 µs 133.46 µs]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) high mild
  11 (11.00%) high severe
bench/geoip2            time:   [87.557 µs 88.734 µs 90.168 µs]
Found 17 outliers among 100 measurements (17.00%)
  6 (6.00%) high mild
  11 (11.00%) high severe

```
