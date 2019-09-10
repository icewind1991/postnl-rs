# PostNL

Rust API client for PostNL consumer api.

## Usage

```rust
let client = PostNL::new(username, password)?;
for package in client.get_packages()?.into_iter() {
    println!(
        "{}({}) - {}",
        package.settings.title, package.key, package.status.delivery_status
    );
}
```

You can get your credentials from [jouw.postnl.nl](https://jouw.postnl.nl).

## Status

Coverage of possible response values is limited to what I can personally retrieve from the api so enums might be missing possible values.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.