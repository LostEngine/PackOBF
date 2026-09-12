# How to optimize PackOBF

PackOBF has a built-in profiler that can be enabled using the `profiling` feature:
```bash
cargo run -p packobf_cli --features profiling -- [packobf options]
```

PackOBF also uses dhat that can be used by enabling debug = 1 in [Cargo.toml](/Cargo.toml) and using
```bash
cargo run -p packobf_cli --features dhat-heap -- [packobf options]
```

> [!Note]
> You can combine both features using `cargo run -p packobf_cli --features dhat-heap --features profiling`
