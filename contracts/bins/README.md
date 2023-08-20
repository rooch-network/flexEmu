## Gen test data with testbin

```shell
../target/release/flexemu --config ../flexemu/config.toml.example gen-state -s 59 ./bins/arith-example 1 10
```

this will generate a dir `step-59` with state and proof at step 59 when running the `arith-example`.

then you can use the `step-59/state-proof.json` file in you move testing like `step_59_test.move` did.
