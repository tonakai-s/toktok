use crate::data::{Data, WebData};

pub async fn execute(data: &Data) {
    match data {
        Data::Web(web_data) => web_execute(web_data).await,
    }
}

pub async fn web_execute(data: &WebData) {
    let response = reqwest::get(data.url()).await;
    match response {
        std::result::Result::Ok(response) => {
            if response.status() == *data.expected_code() {
                println!("Service available with status {}", response.status());
                // Ok(response.status())
            } else {
                println!("Service unavailable with status {}", response.status());
                // bail!(response.status())
            }
        }
        Err(err) => {
            println!("Service unavailable with error {err}");
            // bail!(err)
        }
    }
}
