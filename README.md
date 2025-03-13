# 🧙 Acolyte

> _[Would you like to know the secret to eternal happiness?](https://youtu.be/M_FAL8nVT40?t=25)_

Acolyte is a lightweight resource monitoring tool designed to collect statistics in containerized environments,
particularly Kubernetes.

Acolyte monitors CPU, memory, and GPU utilization and writes the data to JSON files for easy consumption by other
services. It's designed to run alongside your application in the same container and built with compatibility in mind.

Acolyte is configured through environment variables:

* `RUST_LOG`: log level e.g. debug; default: info
* `ACOLYTE_STATS_DIR`: directory where stat files are written; default: /tmp/acolyte/stats
* `ACOLYTE_STAT_INTERVAL_MS`: interval between stats collection in milliseconds; default: 5000
* `ACOLYTE_MAX_STATS_ENTRIES`: maximum number of stat files to keep; default: 12
* `ACOLYTE_CPU_SAMPLE_RATE_MS`: sample window for CPU usage in milliseconds; default: 100
* `SENTRY_DSN`: optional Sentry DSN for error reporting
* `CLUSTER_NAME`: optional cluster identification for Sentry

```shell
# you probably want to run it in the background in your container
./acolyte &

# or attach it to an already running Kubernetes Pod
kubectl cp ./target/x86_64-unknown-linux-musl/release/acolyte my-pod:/tmp/acolyte
kubectl exec my-pod -- sh -c "/tmp/acolyte &"
```

The JSON fields are fairly self-explanatory e.g. `stats-1741860918020.json`:

```json
{
  "time": 1741860918.0206466,
  "num_cpus": 20.0,
  "cpu_usage": 3.5053825547467063,
  "memory_usage_kb": 22802796,
  "memory_total_kb": 65542712,
  "num_gpus": 1,
  "gpu_usage": 0.23,
  "gpu_memory_usage_kb": 50176,
  "gpu_memory_total_kb": 8388608
}
```

## Development

```bash
cargo test

RUST_LOG=debug cargo run
```

## Build

For release, you should build it as a [`musl`](https://en.wikipedia.org/wiki/Musl) static binary:

```shell
rustup target add x86_64-unknown-linux-musl

# https://rust-lang.github.io/rfcs/1721-crt-static.html#specifying-dynamicstatic-c-runtime-linkage
# the flag is enabled by default for this specific target but 🤷
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl

RUST_LOG=debug target/x86_64-unknown-linux-musl/release/acolyte
```

## Release

GitHub will build and host new binaries on every version tag push on the `main` branch.

```shell
git checkout main
git pull origin main

# edit Cargo.toml to update version
git add Cargo.toml
git commit -m 'Become vx.y.z'

git tag vx.y.z
git push origin main vx.y.z
```
