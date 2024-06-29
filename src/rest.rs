use crate::abilities::AbilityName;
use crate::schemas::{Coords, DeployEntitiesRequest, Gamestate, LoginForm};
use crate::services::{tick, ServiceError};
use crate::stores::database::Repository;
use crate::stores::events::Events;
use crate::{services, Repositories};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{async_trait, Form, Json};

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tower_sessions::Session;

use std::sync::mpsc;

pub async fn login(
    State(repo): State<Repositories>,
    session: Session,
    Form(fdata): Form<LoginForm>,
) -> impl IntoResponse {
    match repo
        .db
        .check_user(fdata.username.as_str(), fdata.password.as_str())
        .await
    {
        Ok(roles) => {
            session.insert("roles", &roles).await.unwrap();
            session
                .insert("user", fdata.username.as_str())
                .await
                .unwrap();
            return Redirect::to("/play").into_response();
        }
        Err(error) => return error.into_response(),
    }
}

pub struct AuthenticatedUser {
    pub user_id: String,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            services::ServiceError::NotFound => {
                (StatusCode::NOT_FOUND, "Not found".to_owned()).into_response()
            }
            services::ServiceError::BadRequest(str) => (StatusCode::NOT_FOUND, str).into_response(),
            services::ServiceError::DbError(_) | services::ServiceError::QueueError(_) => {
                tracing::info!("{}", self.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_owned(),
                )
                    .into_response()
            }
            services::ServiceError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_owned()).into_response()
            }
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts.extensions.get::<Session>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract session. Is `SessionManagerLayer` enabled?",
        ))?;
        let user_id = session
            .get::<String>("user")
            .await
            .unwrap_or(None)
            .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized"))?;
        Ok(Self { user_id })
    }
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.clear().await;
    Redirect::to("/login").into_response()
}

pub async fn get_active_game(
    State(repo): State<Repositories>,
    user: AuthenticatedUser,
) -> impl IntoResponse {
    let result = services::get_active_game(repo.db, user.user_id).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn new_game(
    State(repo): State<Repositories>,
    user: AuthenticatedUser,
    axum::extract::Json(scenario_id): axum::extract::Json<i64>,
) -> impl IntoResponse {
    let result = services::new_game(repo.db, user.user_id, scenario_id).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn deploy_entities(
    State(repo): State<Repositories>,
    user: AuthenticatedUser,
    Path(game_id): Path<i64>,
    axum::extract::Json(deploy_entities): axum::extract::Json<DeployEntitiesRequest>,
) -> impl IntoResponse {
    let result =
        services::deploy_entities(repo.db, repo.events, user.user_id, game_id, deploy_entities)
            .await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn get_scenarios(
    State(repo): State<Repositories>,
    _user: AuthenticatedUser,
) -> impl IntoResponse {
    let result = services::get_scenarios(repo.db).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn get_scenario_players(
    State(repo): State<Repositories>,
    _user: AuthenticatedUser,
    Path(scenario_id): Path<i64>,
) -> impl IntoResponse {
    let result = services::get_scenario_players(repo.db, scenario_id).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn get_available_scenario_players(
    State(repo): State<Repositories>,
    _user: AuthenticatedUser,
    Path(game_id): Path<i64>,
) -> impl IntoResponse {
    let result = services::get_available_scenario_players(repo.db, game_id).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn transfer_entity(
    State(repo): State<Repositories>,
    user_from: AuthenticatedUser,
    Path(game_id): Path<i64>,
    Json(user_to): Json<String>,
) -> impl IntoResponse {
    let result = services::transfer_entity(repo.db, game_id, user_from.user_id, user_to).await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn use_ability(
    State(repo): State<Repositories>,
    user: AuthenticatedUser,
    Path((game_id, abilty_name)): Path<(i64, AbilityName)>,
    axum::extract::Json(target): axum::extract::Json<Coords>,
) -> impl IntoResponse {
    let result = services::use_ability(
        repo.db,
        repo.events,
        game_id,
        user.user_id,
        abilty_name,
        target,
    )
    .await;
    match result {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn ws_handler(
    State(repo): State<Repositories>,
    ws: WebSocketUpgrade,
    user: AuthenticatedUser,
    Path(game_id): Path<i64>,
) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, user, repo.events, repo.db, game_id))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(
    socket: WebSocket,
    user: AuthenticatedUser,
    events: Events,
    db: Repository,
    game_id: i64,
) {
    let (sender, receiver) = mpsc::channel::<Gamestate>();
    let register = events.register(game_id, user.user_id.clone(), sender).await;
    if register.is_err() {
        tracing::info!("error: {:?}", register);
        return;
    }
    let _res = tick(events.clone(), db, game_id).await;
    if _res.is_err() {
        tracing::info!("error {:?}", _res.unwrap_err());
    }

    let (sock_sender, sock_receiver) = socket.split();

    tokio::spawn(read_loop(sock_receiver));
    tokio::spawn(write_loop(sock_sender, receiver));
}

async fn read_loop(mut receiver: SplitStream<WebSocket>) {
    loop {
        match receiver.next().await {
            Some(Ok(Message::Close(_))) => {
                tracing::info!("disconnected");
                return;
            }
            Some(Err(_)) => {
                return;
            }
            _ => {}
        }
    }
}

async fn write_loop(
    mut sock_sender: SplitSink<WebSocket, Message>,
    game_receiver: mpsc::Receiver<Gamestate>,
) {
    loop {
        let result = game_receiver.recv();
        if result.is_err() {
            return;
        }
        let res = sock_sender
            .send(Message::Text(
                serde_json::to_string::<Gamestate>(&result.unwrap()).unwrap(),
            ))
            .await;
        if res.is_err() {
            return;
        }
    }
}
