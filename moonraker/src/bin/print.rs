use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let printer = moonraker::Client::new(&args[1])?;
    let path: PathBuf = args[2].parse().unwrap();
    let path: PathBuf = printer.upload_file(&path).await?.item.path.parse().unwrap();
    eprintln!("Uploaded {}", path.display());
    eprintln!("Requesting print");
    printer.print(&path).await?;
    eprintln!("OK");

    Ok(())
}
