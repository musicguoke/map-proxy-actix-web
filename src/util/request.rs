pub mod requests {
    use crate::util::app_error::AppError;
    use reqwest::header::HeaderMap;
    use std::collections::HashMap;
    pub async fn send_tile_request(
        url: &str,
        header_map: Option<HeaderMap>,
        query: Option<HashMap<String, String>>,
    ) -> Result<Vec<u8>, AppError> {
        let client = reqwest::Client::new();
        let user_agent = get_user_agent();
        let mut request = client.get(url);
        if let Some(headers) = header_map {
            request = request.headers(headers);
        }

        if let Some(query_params) = query {
            request = request.query(&query_params);
        }
        let resp = request
            .header(reqwest::header::USER_AGENT, user_agent)
            .send()
            .await
            .map_err(|e| AppError::ApiCallFound(e.to_string()));

        match resp?.bytes().await {
            Ok(bytes) => Ok(bytes.to_vec()),
            Err(e) => Err(AppError::ApiCallFound(e.to_string())),
        }
    }
    pub fn get_user_agent() -> &'static str {
        let random_num: u8 = rand::random_range(1..=6);
        match random_num {
            1 => fake_user_agent::get_chrome_rua(),
            2 => fake_user_agent::get_edge_rua(),
            3 => fake_user_agent::get_firefox_rua(),
            4 => fake_user_agent::get_opera_rua(),
            5 => fake_user_agent::get_safari_rua(),
            _ => {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36"
            }
        }
    }
}
