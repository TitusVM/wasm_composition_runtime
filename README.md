## Issues with this implementation
Running this code (`cargo r`) will result in a linker error: 
```
thread 'main' panicked at src/main.rs:102:63:
called `Result::unwrap()` on an `Err` value: component imports instance `wasi:cli/environment@0.2.0`, but a matching implementation was not found in the linker

Caused by:
    0: instance export `get-environment` has the wrong type
    1: function implementation is missing
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

This is because a crucial step in the instantiation of the `wasmtime` runtime is missing: the addition of the wasi standard imports to the linker (i.e. `wasi:cli/environment@0.2.0`, `wasi:cli/exit@0.2.0`, `wasi:cli/error@0.2.0` etc...). These are listed when looking at `wasmt-tools component wit composition.wasm` which shows the wit of the composed binary:
```
world root {
  import wasi:cli/environment@0.2.0;
  import wasi:cli/exit@0.2.0;
  import wasi:io/error@0.2.0;
  import wasi:io/streams@0.2.0;
  import wasi:cli/stdin@0.2.0;
  import wasi:cli/stdout@0.2.0;
  import wasi:cli/stderr@0.2.0;
  import wasi:clocks/wall-clock@0.2.0;
  import wasi:filesystem/types@0.2.0;
  import wasi:filesystem/preopens@0.2.0;

  export wasi:cli/run@0.2.0;
}
```
Usually, following [`wasmtime` examples](https://github.com/bytecodealliance/wasmtime/blob/main/examples/wasi/main.rs), we need some sort of call to a `add_to_linker` function which I suspect adds these missing imports to the linker. This, however, is not trivial because in our case, there is a Catch-22 when trying to use Wasi and Components (add_to_linker is not compatible with wasmtime::component::Linker, but only with wasmtime::Linker).

