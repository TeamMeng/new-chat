use crate::{
    AppState, AuthOutput,
    error::ErrorOutput,
    handlers::*,
    models::{ChatFile, CreateChat, CreateMessage, ListMessages, SigninUser},
};
use axum::Router;
use chat_core::{Chat, ChatType, ChatUser, Message, User, Workspace};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub(crate) trait OpenApiRouter {
    fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
    paths(
        signin_handler,
        signup_handler,
        list_chat_handler,
        create_chat_handler,
        get_chat_handler,
        list_messages_handler,
        send_message_handler,
        list_chat_users_handler
    ),
    components(schemas(AuthOutput, Chat, ChatType, ChatUser, ChatFile, CreateChat, ChatUser, Message,
         CreateMessage, ListMessages, SigninUser, User, Workspace, ErrorOutput)),
    modifiers(&SecurityAddon),
    tags(
        (name = "Chat", description = "Chat related operations")
    )
)]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            )
        }
    }
}

impl OpenApiRouter for Router<AppState> {
    fn openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    }
}

// eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NjczMjEyNzQsImV4cCI6MTc2NzkyNjA3NCwibmJmIjoxNzY3MzIxMjc0LCJpc3MiOiJjaGF0X3NlcnZlciIsImF1ZCI6ImNoYXRfd2ViIiwiaWQiOjEsIndzX2lkIjoxLCJmdWxsbmFtZSI6IlRlYW1NZW5nIiwiZW1haWwiOiJUZWFtTWVuZ0AxMjMuY29tIiwiY3JlYXRlZF9hdCI6IjIwMjUtMTItMzFUMDM6NDc6MzMuMTEyNzQxWiJ9.DO2CcQ7g_jvi2S6B8h9ceSx84s_5dMHaTjcoUO5au5my2LcqYGJkw-sKfu0RQqtare0HXN0ftdxDUymVXJqaDA
