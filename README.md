`cargo watch -x check -x test -x run`

`cargo install cargo-tarpaulin`



```
brew install michaeleisel/zld/zld  
```

```
rustup toolchain list
rustup component add clippy
rustup component add rustfmt
cargo install cargo-audit
cargo audit

cargo +nightly expand
```

```
set -eo pipefail

```


Cargo install

```
Nothing or a caret (^) means "at least this version, until the next incompatible version".

A tilde (~) means "at least this version, until (but excluding) the next minor/major release". That is, ~1.2.3 will accept 1.2.X where X is at least 3, ~1.2 will accept 1.2.*, and ~1 will accept 1.*.*.

A wildcard (*) means "anything that looks like this". That is, 1.2.* will accept 1.2.anything (1.2.0, 1.2.7-beta, 1.2.93-dev.foo, etc. but not 1.3.0).

Inequalities (>=, >, <, =) mean the obvious: the version Cargo uses must satisfy the given inequality.

```



sqlx

```
cargo install --version="~0.6" sqlx-cli --no-default-features --features rustls,postgres

export DATABASE_URL=postgres://postgres:password@localhost:5432/newsletter
sqlx migrate add create_subscriptions_table 
```


docker stop

```
docker stop $(docker ps -q --filter "name=newsletter-db")       
```


Logger - env_logger

```
RUST_LOG=debug cargo run

RUST_LOG=zero2prod cargo run #Filter out all records emitted by our dependencies
```


Cargo remove unnecessary dependencies

```
cargo install cargo-udeps

cargo +nightly udeps

```


Bunyan formatter

```
cargo install bunyan
TEST_LOG=true cargo test | bunyan
```



Disable inline hints

```
{
  "editor.inlayHints.enabled": "offUnlessPressed"
}
```
Then you can toggle them with `Ctrl + Alt` pressed.


**Sqlx offline mode**

```
cargo sqlx prepare -- --lib 

```

Docker entry

```
ENV SQLX_OFFLINE=true
```

```
docker build -t zero2prod .   
```