use std::time::Duration;
use thirtyfour::{prelude::*, ChromeCapabilities};
use futures::future::join_all;

pub struct Mlb {
    pub driver: WebDriver,
    pub base_url: String,

}

impl Mlb {
    pub async fn new(headless: bool) -> WebDriverResult<Mlb> {
        let caps = if headless {
            let mut caps = ChromeCapabilities::new();
            caps.set_headless()?;
            caps
        } else {
            ChromeCapabilities::new()
        };

        let driver = WebDriver::new("http://localhost:9515", caps).await?;
        let base_url = "https://www.mlb.com/stats/".to_string();
        driver.goto(&base_url).await?;
        
        Ok(Mlb { driver, base_url })
    }

    pub async fn close_banner(&self) -> WebDriverResult<()> {
        /* Query the page for the banner close btn element
        - wait for 6 seconds and try every 1 second
         */
        let banner_close_btn = self.driver.query(By::ClassName("banner-close-button")).wait(Duration::from_secs(8), Duration::from_secs(1)).first().await?;
        banner_close_btn.wait_until().clickable().await?;
        // banner_close_btn.click().await?;
        match banner_close_btn.click().await {
            Ok(_) => println!("Banner closed"),
            Err(_) => println!("Banner not found"),
        }
        Ok(())

    }

    pub async fn get_std_columns(&self) -> WebDriverResult<Vec<String>> {
        let active_tbl = self.get_active_table().await?;
        match active_tbl.to_lowercase().as_str() {
            "standard" => {
                let cols = self.gather_column_headers().await?;
                Ok(cols)
            },
            _ => {
                self.switch_table().await?;
                let cols = self.gather_column_headers().await?;
                Ok(cols)
            }
        }
    }

    pub async fn gather_expanded_columns(&self) -> WebDriverResult<Vec<String>> {
        let active_tbl = self.get_active_table().await?;
        match active_tbl.to_lowercase().as_str() {
            "expanded" => {
                let cols = self.gather_column_headers().await?;
                Ok(cols)
            },
            _ => {
                self.switch_table().await?;
                let cols = self.gather_column_headers().await?;
                Ok(cols)
            }
        }
    }

    pub async fn get_active_table(&self) -> WebDriverResult<String> {
        let active_tbl = self.driver.query(By::Css(".stats-navigation div.group-secondary button.selected")).first().await?;
        let tbl_type = active_tbl.text().await?;
        // println!("Table Type: {}", tbl_type);
        Ok(tbl_type)
    }

    pub async fn switch_table(&self) -> WebDriverResult<()> {
        let switch = self.driver.query(By::Css(".stats-navigation div.group-secondary button:not(.selected)")).first().await?;
        switch.wait_until().clickable().await?;
        switch.click().await?;
        println!("Switched to: {}", switch.text().await?);
        Ok(())
    }

    pub async fn gather_column_headers(&self) -> WebDriverResult<Vec<String>>{
        let cols = self.driver.query(By::Css("table thead tr th button abbr")).all().await?;
        let mut col_headers = Vec::new();
        for col in cols {
            let text = col.text().await?;
            // println!("Column: {}", text);
            col_headers.push(text.to_lowercase());
        }
        col_headers.insert(1, String::from("pos"));
        Ok(col_headers)
    }

    pub async fn gather_rows(&self) -> WebDriverResult<Vec<Vec<String>>> {
        let rows = self.driver.query(By::Css("table tbody tr")).all().await?;
        let mut row_data = Vec::new();
        
        for row in rows {
            let mut row_container = Vec::new();
            let player_name = row.query(By::Css("a.bui-link")).first().await?;
            let player_name = player_name.attr("aria-label").await?.unwrap_or("".to_string());
            row_container.push(player_name);


            let position = row.query(By::Css("div[class^='position']")).first().await?;
            let position = position.text().await?;
            row_container.push(position);

            let stats = row.query(By::Css("td")).all().await?;
            for stat in stats {
                let stat = stat.text().await?;
                row_container.push(stat);
            }
            row_data.push(row_container);
            
        }
        
        Ok(row_data)
    }

    pub async fn gather_rows_fifo(&self) -> WebDriverResult<Vec<Vec<String>>> {
        let rows = self.driver.query(By::Css("table tbody tr")).all().await?;
        let mut row_data = Vec::new();
        for row in rows {
            // let mut fifo = FuturesOrdered::new();
            let mut row_container = Vec::new();
            let player_name = row.query(By::Css("a.bui-link")).first().await?;
            let player_name = player_name.attr("aria-label").await?.unwrap_or("".to_string());
            // fifo.push_front(player_name);
            row_container.push(player_name);
            let position = row.query(By::Css("div[class^='position']")).first().await?;
            let position = position.text().await?;
            // fifo.push_back(position);
            row_container.push(position);
            // await to gather all the stats
            let stats = row.query(By::Css("td")).all().await?;
            // Create a vec to hold the individual table cell "stats" text futures 
            let mut stats_fut = vec![];
            // iterate over the stats and map the text future to the stats_fut vec
            stats_fut.extend(stats.iter().map(|stat| stat.text()));

            let stats = join_all(stats_fut).await;
            
            for stat in stats {
                if let Ok(s) = stat {
                    row_container.push(s);
                }
            }

            row_data.push(row_container);
        }
        
        Ok(row_data)

    }

    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn column_test() -> WebDriverResult<()> {
        let mlb = Mlb::new(true).await?;
        mlb.close_banner().await?;
        let cols = mlb.get_std_columns().await?;
        assert_eq!(vec!["player", "pos", "team", "g", "ab", "r", "h", "2b", "3b", "hr", "rbi", "bb", "so", "sb", "cs", "avg", "obp", "slg", "ops"], cols);
        
        let exp_cols = mlb.gather_expanded_columns().await?;
        assert_eq!(vec!["player", "pos", "team", "pa", "hbp", "sac", "sf", "gidp", "go/ao", "xbh", "tb", "ibb", "babip", "iso", "ab/hr", "bb/k", "bb%", "so%"], exp_cols);

        Ok(())
    }




    
}
