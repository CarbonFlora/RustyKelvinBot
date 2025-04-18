use crate::RustyKelvinBot;

impl RustyKelvinBot {
    pub async fn test(self) {
        let response = reqwest::get("https://jsonplaceholder.typicode.com/posts")
            .await
            .expect("Failed to query api for jsonplaceholder.")
            .text()
            .await
            .expect("Failed to parse api package as text.");
        self.send_message(response).await;
    }
}
