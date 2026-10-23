# Test fixtures

These are tiny, dependency-free binaries used by `tests/cli.rs`:

| File | Format | Exported names |
| --- | --- | --- |
| `demo.dll` | PE32+ DLL (x86-64) | `?add@demo@@YAHHH@Z`, `plain_c_export` |
| `libdemo.so` | ELF64 shared object (x86-64) | `_ZN4demo3addEii`, `plain_c_export` |
| `libdemo.dylib` | Mach-O 64 dylib (arm64) | `__Z8demo_addii`, `_plain_c_export` |

They are built from `src/` with `build.sh`. The ELF and PE files have no libc
dependency, so they can be linked with the `rust-lld` that ships with Rust.

```sh
./build.sh
```

The Mach-O fixture is only rebuilt on macOS. The checked-in binaries are
deliberately committed so the test suite does not need a cross-compilation
toolchain.
