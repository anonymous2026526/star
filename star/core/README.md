# star-core

`star-core` is the core library of STAR.

## Test

```sh
cargo test
```

## Benchmark

### enclave

For the `enclave` benchmark, select the Fortanix SGX target.

```sh
cargo bench --bench enclave --target x86_64-fortanix-unknown-sgx
```

### user

For the `user` benchmark, select the normal target.

```sh
cargo bench --bench user
```

### Communication

Run the communication benchmark as follows:

```sh
cargo run --bin comm --release
```

## Side-channel check

The files for side-channel checking are located in `analysis`.

### Environment setup

`shell.nix` is used to set up the environment.

```sh
cd analysis
nix-shell
```

### Run the check

Run the following command in the `analysis` directory. It will output a list of potentially dangerous branches and memory accesses.

```sh
sh run.sh
```

### Inspect the target assembly

Write the address shown in the `run.sh` output into `obdump.sh`, then run it to display the corresponding assembly.

```sh
sh obdump.sh
```
