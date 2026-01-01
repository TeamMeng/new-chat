use anyhow::Result;
use chat_core::{Chat, ChatType, Message};
use chat_server::{AppState, get_router};
use futures::StreamExt;
use reqwest::{
    Client, StatusCode,
    multipart::{Form, Part},
};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, time::sleep};

const WILD_ADDR: &str = "0.0.0.0:0";

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

struct NotifyServer;

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: Client,
}

#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (tdb, state) = AppState::new_for_test().await?;
    let db_url = tdb.url();
    let chat_server = ChatServer::new(state).await?;
    NotifyServer::new(db_url, &chat_server.token).await?;
    let chat = chat_server.create_chat().await?;
    chat_server.create_message(chat.id as _).await?;
    chat_server.upload().await?;
    sleep(Duration::from_secs(1)).await;
    Ok(())
}

impl NotifyServer {
    async fn new(db_url: String, token: &str) -> Result<Self> {
        let mut config = notify_server::AppConfig::load()?;
        config.server.db_url = db_url;
        let app = notify_server::get_router(config).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),
                    Ok(Event::Message(message)) => match message.event.as_str() {
                        "NewChat" => {
                            let chat: Chat = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(chat.name, Some("test".to_string()));
                            assert_eq!(chat.members, vec![1, 2]);
                            assert_eq!(chat.r#type, ChatType::PrivateChannel);
                        }

                        "NewMessage" => {
                            let msg: Message = serde_json::from_str(&message.data).unwrap();
                            assert_eq!(msg.content, "Hello World!");
                            assert_eq!(msg.files.len(), 0);
                            assert_eq!(msg.sender_id, 1);
                        }
                        _ => {
                            panic!("unexpected event: {:?}", message);
                        }
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                        es.close();
                    }
                }
            }
        });

        Ok(NotifyServer)
    }
}

impl ChatServer {
    async fn new(state: AppState) -> Result<Self> {
        let app = get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;

        let addr = listener.local_addr()?;

        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let client = Client::new();

        let mut ret = Self {
            addr,
            token: "".to_string(),
            client,
        };

        ret.token = ret.signin().await?;

        Ok(ret)
    }

    async fn signin(&self) -> Result<String> {
        let res = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(
                r#"{
                    "email": "Test@123.com",
                    "password": "123456"
                }"#,
            )
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);
        let ret: AuthToken = res.json().await?;
        Ok(ret.token)
    }

    async fn create_chat(&self) -> Result<Chat> {
        let res = self
            .client
            .post(format!("http://{}/api/chats", self.addr))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(
                r#"
                {
                    "name": "test",
                    "members": [1, 2],
                    "public": false
                }
            "#,
            )
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);
        let chat: Chat = res.json().await?;
        assert_eq!(chat.name, Some("test".to_string()));
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        Ok(chat)
    }

    async fn create_message(&self, chat_id: u64) -> Result<()> {
        let res = self
            .client
            .post(format!("http://{}/api/chats/{}", self.addr, chat_id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.token))
            .body(
                r#"
                {
                    "content": "Hello World!",
                    "files": []
                }
            "#,
            )
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);
        let msg: Message = res.json().await?;
        assert_eq!(msg.content, "Hello World!");
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.chat_id, chat_id as i64);
        assert_eq!(msg.files, Vec::<String>::new());

        Ok(())
    }

    async fn upload(&self) -> Result<()> {
        // upload file
        let data = include_bytes!("../Cargo.toml");
        let files = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;

        let form = Form::new().part("file", files);

        let res = self
            .client
            .post(format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);
        let vec: Vec<String> = res.json().await?;
        assert!(!vec[0].is_empty());

        Ok(())
    }
}
