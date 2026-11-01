# Initial measurements

Measured 2026-11-01, Linux x86-64 sandbox, kernel 6.1.158, Rust 1.88.0 release build:

```sh
cargo build --workspace --release --locked
python3 scripts/benchmark.py --entries 1000 --trials 11
```

| CLI smoke workload | p50 ms | p95 ms | p99 ms |
|---|---:|---:|---:|
| `--help` process startup | 1.479 | 1.936 | 1.936 |
| 1000-entry directory listing (paged, including process) | 7.100 | 7.457 | 7.457 |
| Approx. 8 MiB streaming copy + complete verification + fsync | 19.157 | 21.608 | 21.608 |

These are actual single-run observations, **not throughput targets or product claims**.
With 11 samples, p95/p99 both select the maximum and are not statistically robust
tail estimates. Payload is 8,388,606 bytes; directory entries are 1 KiB files. Fixtures
are private and temporary; copies are removed between samples. OS caches are warm;
process startup and CLI formatting are included. No cache flush, pinned CPU, device
model/control, GUI startup, WebView frame latency, independent memory profiling,
Android/Windows result, cold-cache comparison or before/after optimization is claimed.

The current directory algorithm bounds work at 50,000 entries but collects/sorts
before paging and rescans for later pages. It is not yet the virtualized/incremental
browser in the roadmap. Future benchmarks must exercise deep trees, many small
files, sparse/large files, slow volumes, concurrent operations and UI responsiveness,
then compare repeatable distributions on controlled native hardware.
