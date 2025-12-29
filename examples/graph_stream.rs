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
