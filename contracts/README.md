## Contracts

The folder contains Move modules used in layer2 to verify layer2 instructions.

### Compile

Download move-cli from [movelang](https://github.com/move-language/move)

```
$ move build -d
```

### Keys

`./dev.key` is the private key corresponding to the dev-address of omo.

You can use the key to deploy contracts if you don't want to change the dev-address in Move.toml.
And dont use it in production!