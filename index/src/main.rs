#[tokio::main]
async fn main() {
    let http_client = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let database = "pages";
    let design_document = "search";
    let view = "by_keyword";
    let view_map_src = include_str!("view/map.js").replace("\n", "");

    let request_params = format!(
        "http://admin:fixme@couchdb:5984/{}/_design/{}",
        database,
        design_document,
    );

    let request_object = format!(
        r#"
        {{
            "views": {{
                "{}": {{
                    "map", "{}",
                }}
            }},
            "language": "javascript"
        }}
        "#,
        view,
        view_map_src,
    );

    println!("{}", request_object);

    let res = http_client
        .put(&request_params)
        .json(&request_object)
        .send()
        .await.unwrap();

    println!("{:?}", res.text().await);
}
