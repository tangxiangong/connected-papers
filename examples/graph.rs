use connected_papers::ConnectedPapers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ConnectedPapers::with_api_key("TEST_TOKEN");

    let response = client
        .get_graph("9397e7acd062245d37350f5c05faf56e9cfae0d6", false)
        .await?;

    println!("{response:#?}");
    Ok(())
}
