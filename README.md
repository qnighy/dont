## `dont` command: swiss army knife for everything you do not want to do

### Installation

```
cargo install dont
```

### Usage

```
dont echo "hello world"
```

It doesn't print `hello world`.

```
dont do-release-upgrade
```

It doesn't upgrade your operating system.

```
dont ls
```

It doesn't list the contents of the current directory.

```
dont dont echo "hello world"
```

It doesn't follow your second `dont`. That means... uh oh.

### Contributing

If you find cases where `dont` doesn't properly negate your intentions, feel free to submit a pull request. Be sure to include a test case.

Check your code by executing the following:

```
cargo test
cargo fmt
cargo clippy
```
