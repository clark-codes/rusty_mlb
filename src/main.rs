use web_crawler::Mlb;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = std::time::Instant::now();
    let mlb = Mlb::new(true).await?;
    
    println!("Current URL: {}", mlb.driver.current_url().await?);
    mlb.close_banner().await?;
    let cols = mlb.get_std_columns().await?;
    println!("Columns: {:?}", cols);
    let rows = mlb.gather_rows_fifo().await?;
    println!("Rows: {:?}", rows);

    println!("Time elapsed: {:?}", start.elapsed());
    mlb.driver.quit().await?;
    Ok(())
}