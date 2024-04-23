# MaxMind DB

This library reads the MaxMind DB format, including the GeoIP2 and GeoLite2 databases.

## Features
- lightweight
- less dependencies

## Mmap
Mmap will use less memory than in-memory implementation.

## Bench
```text
bench/in-memory         time:   [71.337 µs 71.562 µs 71.856 µs]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
bench/mmap              time:   [72.232 µs 73.072 µs 73.882 µs]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
bench/geoip2            time:   [88.798 µs 89.124 µs 89.487 µs]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

```
