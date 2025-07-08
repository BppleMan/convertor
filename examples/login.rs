use reqwest::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let response = client
        .post("https://www.blnew.com/proxy/passport/auth/login/")
        .form(&[("email", "1286363484@qq.com"), ("password", "aa206462546918093")])
        .send()
        .await;
    println!("{}", response.unwrap().status());
}
