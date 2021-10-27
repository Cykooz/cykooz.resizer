# Notes for developers

## Install dependencies

- Install Rust with help of `rustup` (https://rustup.rs/)
- Install Pythons dependencies:
  ```shell
  python3 -m pip install .[dev] 
  ```

## Build Rust library

### Debug version

```shell
maturin develop
```

### Optimized version

```shell
maturin develop --release --strip
```

## Run tests

```shell
pytest --benchmark-skip
```

## Run benchmark

```shell
pytest -s tests/test_benchmark.py
```

## Build release wheels and sdist

```shell
maturin build --release --strip
```
