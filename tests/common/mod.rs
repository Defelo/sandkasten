use sandkasten_client::BlockingSandkastenClient;

pub fn client() -> BlockingSandkastenClient {
    BlockingSandkastenClient::new(
        option_env!("TARGET_HOST")
            .unwrap_or("http://127.0.0.1:8000")
            .parse()
            .unwrap(),
    )
}
