# Connected Papers

A Rust client for [Connected Papers](https://www.connectedpapers.com/) integrated with [Semantic Scholar](https://www.semanticscholar.org/) utilities, inspired by the [Official Python Client](https://github.com/ConnectedPapers/connectedpapers-py).

## Quick Start

### Basic Usage

```rust
use connected_papers::ConnectedPapers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ConnectedPapers::with_api_key("your_api_key_here");
    
    let response = client
        .get_graph("9397e7acd062245d37350f5c05faf56e9cfae0d6", false)
        .await?;
    
    println!("{response:#?}");
    Ok(())
}
```

### Streaming

```rust
use connected_papers::ConnectedPapers;
use futures::StreamExt;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ConnectedPapers::with_api_key("TEST_TOKEN");

    let mut stream =
        client.get_graph_stream("9397e7acd062245d37350f5c05faf56e9cfae0d6", false, true);

    let mut stdout = io::stdout();
    while let Some(result) = stream.next().await {
        match result {
            Ok(response) => {
                writeln!(stdout, "{response:#?}")?;
                stdout.flush()?;
            }
            Err(e) => {
                writeln!(stdout, "Error: {e}")?;
                stdout.flush()?;
                return Err(e.into());
            }
        }
    }

    Ok(())
}
```

### Remaining Usages

```rust
let remaining = client.get_remaining_usages().await?;
println!("Remaining API calls: {}", remaining);
```

### Free Access Papers

```rust
let papers = client.get_free_access_papers().await?;
println!("{papers:#?}");
```

## License

Licensed under either of:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
