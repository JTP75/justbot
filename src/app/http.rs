use std::{fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use axum::{Router, extract, routing, http::status::StatusCode};
use chrono::Local;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{net::TcpListener, signal};
use derive_builder::Builder;

use crate::{
    app::{connection_manager::ConnectionManager, tool_manager::ToolManager}, 
    common::config, connection::anthropic_client::{AnthropicToolDefinition, Message, MessagesResponse, ToolResultContentBlock}
};

// lazy mutex for server state
// ONLY ALLOW IDEMPOTENT MUT OPS!!! (i.e. no POST)

pub struct AppState {
    pub collection_name: Option<String>,
    pub motd: (chrono::NaiveDate, Option<String>)
}

pub static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(
    || Mutex::new(AppState { 
        collection_name: None,
        motd: {
            let filename: String = crate::common::config
                ::get_config("bot_config.json","motd_filename")
                .expect("failed to get config");
            let value = fs::read_to_string(config::PROJECT_DIRS.data_dir().join(filename));
            match value {
                Ok(value) => serde_json::from_str(&value)
                    .unwrap_or((Local::now().date_naive(), None)),
                Err(_) => (Local::now().date_naive(), None)
            }
        }
    }));

// helper macros

macro_rules! endpoint_route_post {
    ($cm:expr, $tm:expr, $endpoints:expr) => {
        routing::post({
            let cm = Arc::clone(&$cm);
            let tm = Arc::clone(&$tm);
            let endpoints = Arc::clone(&$endpoints);
            |extract::Json(req): extract::Json<HttpRequest>| async move {
                let endpoint = endpoints.iter()
                    .find(|e| e.name() == req.endpoint)
                    .ok_or("endpoint not found");
                if let Err(e) = endpoint {
                    return (
                        StatusCode::BAD_REQUEST, 
                        extract::Json(HttpResponse { status: "error".into(), data: json!({"error": e.to_string()}) })
                    );
                }
                match endpoint.unwrap().handle(&cm, &tm, req.data).await {
                    Ok(data) => (
                        StatusCode::OK, 
                        extract::Json(HttpResponse { status: "ok".into(), data })
                    ),
                    Err(e) => (
                        StatusCode::BAD_REQUEST, 
                        extract::Json(HttpResponse { status: "error".into(), data: json!({"error": e.to_string()}) })
                    )
                }
            }
        })
    };
}

macro_rules! endpoint_route_put {
    ($cm:expr, $tm:expr, $endpoints:expr) => {
        routing::post({
            let cm = Arc::clone(&$cm);
            let tm = Arc::clone(&$tm);
            let endpoints = Arc::clone(&$endpoints);
            |extract::Json(req): extract::Json<HttpRequest>| async move {
                let endpoint = endpoints.iter()
                    .find(|e| e.name() == req.endpoint)
                    .ok_or("endpoint not found");
                if let Err(e) = endpoint {
                    return (
                        StatusCode::BAD_REQUEST, 
                        extract::Json(HttpResponse { status: "error".into(), data: json!({"error": e.to_string()}) })
                    );
                }
                match endpoint.unwrap().handle(&cm, &tm, req.data).await {
                    Ok(data) => (
                        StatusCode::OK, 
                        extract::Json(HttpResponse { status: "ok".into(), data })
                    ),
                    Err(e) => (
                        StatusCode::BAD_REQUEST, 
                        extract::Json(HttpResponse { status: "error".into(), data: json!({"error": e.to_string()}) })
                    )
                }
            }
        })
    };
}

macro_rules! endpoint_route_get {
    ($cm:expr, $tm:expr, $endpoints:expr) => {
        routing::get({
            let cm = Arc::clone(&$cm);
            let tm = Arc::clone(&$tm);
            let endpoints = Arc::clone(&$endpoints);
            |extract::Query(req): extract::Query<HttpRequest>| async move {
                let endpoint = endpoints.iter()
                    .find(|e| e.name() == req.endpoint)
                    .ok_or("endpoint not found");
                if let Err(e) = endpoint {
                    return (
                        StatusCode::BAD_REQUEST, 
                        extract::Json(HttpResponse { status: "error".into(), data: json!({"error": e.to_string()}) })
                    );
                }
                match endpoint.unwrap().handle(&cm, &tm, json!({})).await {
                    Ok(data) => (
                        StatusCode::OK, 
                        extract::Json(HttpResponse { status: "ok".into(), data })
                    ),
                    Err(e) => (
                        StatusCode::BAD_REQUEST, 
                        extract::Json(HttpResponse { status: "error".into(), data: json!({"error": e.to_string()}) })
                    )
                }
            }
        })
    }
}

// trait/helper struct hell

#[async_trait::async_trait]
pub trait HttpEndpoint: Send + Sync {
    fn name(&self) -> &str;
    async fn handle(&self, cm: &Arc<ConnectionManager>, tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>>;
}

#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
pub struct HttpRequest {
    pub endpoint: String,
    #[builder(default = json!(0))]
    pub data: Value
}

#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
pub struct HttpResponse {
    #[builder(default = "error".into())]
    pub status: String,
    #[builder(default = json!(0))]
    pub data: Value
}

// struct hell

// ======== send_message
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
pub struct SendMessageParams {
    messages: Vec<Message>,
    #[builder(default = "{{EMPTY SYSTEM PROMPT}}".into())]
    sys_prompt: String,
    #[builder(default = false)]
    use_tools: bool,
    #[builder(default = None)]
    model: Option<String>,
    #[builder(default = 0.75f64)]
    randomness: f64
}
pub struct SendMessageEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for SendMessageEndpoint {
    fn name(&self) -> &str { "send_message" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling send_message request: {:?}", input);
        let args = serde_json::from_value::<SendMessageParams>(input)?;
        let tools = if args.use_tools { Some(&tm.get_tools_as_tooldefs()) } else { None };
        let response = cm.chat_client.call_model(
            &args.messages, &args.sys_prompt, tools, args.model, args.randomness).await;
        match response {
            Ok(response) => Ok(json!(response)),
            Err(e) => {
                log::error!("Error in call_model call: {e}");
                Err(e)
            }
        }
    }
}

// ======== get_tpm
pub struct GetTpmEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for GetTpmEndpoint {
    fn name(&self) -> &str { "get_tpm" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling get_tpm request: {:?}", input);
        Ok(json!(cm.chat_client.usage_monitor.tpm()))
    }
}

// ======== get_max_tpm
pub struct GetMaxTpmEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for GetMaxTpmEndpoint {
    fn name(&self) -> &str { "get_max_tpm" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling get_max_tpm request: {:?}", input);
        Ok(json!(cm.chat_client.usage_monitor.max_tpm()))
    }
}

// ======== list_tools
pub struct ListToolsEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for ListToolsEndpoint {
    fn name(&self) -> &str { "list_tools" }
    async fn handle(&self, _cm: &Arc<ConnectionManager>, tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling list_tools request: {:?}", input);
        let tools = tm.get_tools_as_tooldefs();
        Ok(json!(tools))
    }
}

// ======== execute_tool
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
pub struct ExecuteToolParams {
    name: String,
    input: Value,
}
pub struct ExecuteToolEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for ExecuteToolEndpoint {
    fn name(&self) -> &str { "execute_tool" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling execute_tool request: {:?}", input);
        let args = serde_json::from_value::<ExecuteToolParams>(input)?;
        let result = tm.execute_tool(cm, &args.name, &args.input)?;
        Ok(json!(result))
    }
}

// ======== embed_files
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
pub struct EmbedFilesParams {
    collection_name: String,
    file_paths: Vec<PathBuf>
}
pub struct EmbedFilesEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for EmbedFilesEndpoint {
    fn name(&self) -> &str { "embed_files" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling embed_files request: {:?}", input);
        let args = serde_json::from_value::<EmbedFilesParams>(input)?;
        cm.embed_files(&args.collection_name, 
            args.file_paths.iter().map(|path| path.as_path()).collect()).await?;
        Ok(json!(null))
    }
}

// ======== query_vdb
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
pub struct QueryVdbParams {
    collection_name: String,
    query: String,
    #[builder(default = None)]
    search_limit: Option<u64>
}
pub struct QueryVdbEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for QueryVdbEndpoint {
    fn name(&self) -> &str { "query_vdb" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling query_vdb request: {:?}", input);
        let args = serde_json::from_value::<QueryVdbParams>(input)?;
        let res = cm.query_vdb(&args.collection_name, &args.query, args.search_limit).await?;
        Ok(json!(res))
    }
}

// ======== list_collections
pub struct ListCollectionsEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for ListCollectionsEndpoint {
    fn name(&self) -> &str { "list_collections" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling list_collections request: {:?}", input);
        let collections = cm.vdb_client.list_collections().await?;
        Ok(json!(collections))
    }
}

// ======== add_collection
pub struct AddCollectionEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for AddCollectionEndpoint {
    fn name(&self) -> &str { "add_collection" }
    async fn handle(&self, cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling add_collection request: {:?}", input);
        let collection_name = serde_json::from_value::<String>(input)?;
        cm.vdb_client.add_collection(&collection_name).await?;
        Ok(json!(null))
    }
}

// ======== set_collection
pub struct SetCollectionEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for SetCollectionEndpoint {
    fn name(&self) -> &str { "set_collection" }
    async fn handle(&self, _cm: &Arc<ConnectionManager>, _tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling set_collection request: {:?}", input);
        let collection_name = serde_json::from_value::<String>(input)?;
        APP_STATE.lock().unwrap().collection_name = Some(collection_name);
        Ok(json!(null))
    }
}

// ======== health
pub struct HealthEndpoint;
#[async_trait::async_trait]
impl HttpEndpoint for HealthEndpoint {
    fn name(&self) -> &str { "health" }
    async fn handle(&self, _cm: &Arc<ConnectionManager>, tm: &Arc<ToolManager>, input: Value) 
    -> Result<Value, Box<dyn std::error::Error>> {
        log::debug!("Handling health request: {:?}", input);

        let mcp_tools = tm.get_mcp_tooldefs()
            .iter()
            .map(|td| td.name.clone())
            .collect::<Vec<_>>()
            .join("\t\n");

        let anthropic_status = "todo";
        let qdrant_status = "todo";
        let embedding_service = "todo"; // "local" or "voyage"
        let embedding_status = "todo";

        // if cm.chat_client.
        
        log::info!("Health check called\nMCP Tools:\n\t{}\nAnthropic:\n\t{}\nQdrant:\n\t{}\nEmbedding Service ({}):\n\t{}", 
            mcp_tools,
            anthropic_status,
            qdrant_status,
            embedding_service, embedding_status
        );

        Ok(json!(null))
    }
}

// http server

pub struct HttpServer {
    router: Option<Router>,
    listener: Option<TcpListener>,
    endpoints: Vec<Arc<dyn HttpEndpoint>>,
    host: String,
    port: u64,
}

impl HttpServer {
    pub fn new() -> Self {
        let host = crate::common::config
            ::get_config("bot_config.json", "backend_host")
            .expect("failed to retrieve config");
        let port = crate::common::config
            ::get_config("bot_config.json", "backend_port")
            .expect("failed to retrieve config");
        
        let endpoints: Vec<Arc<dyn HttpEndpoint>> = vec![

            // health
            Arc::new(HealthEndpoint),
            
            // chat
            Arc::new(SendMessageEndpoint),
            Arc::new(GetTpmEndpoint),
            Arc::new(GetMaxTpmEndpoint),

            // tools
            Arc::new(ListToolsEndpoint),
            Arc::new(ExecuteToolEndpoint),

            // connect
            Arc::new(EmbedFilesEndpoint),
            Arc::new(QueryVdbEndpoint),
            Arc::new(ListCollectionsEndpoint),
            Arc::new(AddCollectionEndpoint),

            // state
            Arc::new(SetCollectionEndpoint),

            // ...
        ];

        Self { router: None, listener: None, endpoints, host, port }
    }

    pub fn init_router(&mut self, cm: Arc<ConnectionManager>, tm: Arc<ToolManager>) -> Result<(), Box<dyn std::error::Error>> {
        
        let cm = Arc::clone(&cm);
        let tm = Arc::clone(&tm);
        let endpoints = Arc::new(self.endpoints.clone());

        let tool_routes = Router::new()
            .route("/list_tools", endpoint_route_get!(cm, tm, endpoints))
            .route("/execute_tool", endpoint_route_post!(cm, tm, endpoints));
        let chat_routes = Router::new()
            .route("/message", endpoint_route_post!(cm, tm, endpoints))
            .route("/get_tpm", endpoint_route_get!(cm, tm, endpoints))
            .route("/get_max_tpm", endpoint_route_get!(cm, tm, endpoints));
        let connect_routes = Router::new()
            .route("/add_collection", endpoint_route_post!(cm, tm, endpoints))
            .route("/list_collections", endpoint_route_get!(cm, tm, endpoints))
            .route("/embed_files", endpoint_route_post!(cm, tm, endpoints))
            .route("/query_vdb", endpoint_route_post!(cm, tm, endpoints))
            .nest("/chat", chat_routes);
        let state_routes = Router::new()
            .route("/set_collection", endpoint_route_put!(cm, tm, endpoints));
        let router = Router::new()
            .route("/health", endpoint_route_get!(cm, tm, endpoints))
            .nest("/tools", tool_routes)
            .nest("/connect", connect_routes)
            .nest("/state", state_routes);

        self.router = Some(router);
        Ok(())
    }

    pub fn addr(&self) -> String { format!("{}:{}", self.host, self.port) }

    pub async fn init_tcp_listener(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(self.addr())
            .await.map_err(|e| format!("Failed to create listener: {}", e))?;

        self.listener = Some(listener);
        Ok(())
    }

    pub async fn serve(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(router) = self.router.take() {
            if let Some(listener) = self.listener.take() {
                log::debug!("Backend hosted on {}", self.addr());
                axum::serve(listener, router)
                    .with_graceful_shutdown(shutdown_signal())
                    .await.map_err(|e| format!("Server failed: {}", e))?;
                return Ok(());
            }
        }
        Err("Router or TCP Listener not initialized".into())
    }
}

/// uses
/// 
/// connection manager
/// 
/// - embed_files
/// - query_vdb
/// - add_collection
/// - list_collections
/// - chat/call_model
/// - chat/get_tpm
/// - chat/get_max_tpm
/// 
/// tool manager
/// 
/// - get_tooldefs
/// - execute_tool

// http client

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    host: String,
    port: u64
}

impl HttpClient {
    pub fn new() -> Self {
        let host = crate::common::config
            ::get_config("bot_config.json", "backend_host")
            .expect("failed to retrieve config");
        let port = crate::common::config
            ::get_config("bot_config.json", "backend_port")
            .expect("failed to retrieve config");
        Self { client: Client::new(), host, port }
    }

    pub fn addr(&self) -> String { format!("{}:{}", self.host, self.port) }

    async fn get(&self, url: String, value: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let response = self.client
            .get(url)
            .query(&value)
            .send().await?;
        if let Err(e) = response.error_for_status_ref() {
            log::error!("Get request failed: {e}");
            log::error!("Body: {value}");
            Err(Box::new(e))
        } else {
            let response_body = response.json().await?;
            Ok(response_body)
        }
    }

    async fn post(&self, url: String, value: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let response = self.client
            .post(url)
            .header("content-type", "application/json")
            .json(&value)
            .send().await?;

        let status = response.status();
        let response_body = response.json::<Value>().await?;
        
        if !status.is_success() {
            log::error!("Post request failed with status: {status}");
            log::error!("Response body: {}", serde_json::to_string_pretty(&response_body).unwrap());
            return Err(format!("Request failed with status: {status}").into());
        }
        
        Ok(response_body)
    }

    pub fn is_healthy(&self) -> bool {
        let health = tokio::runtime::Runtime::new().expect("failed to create runtime")
            .block_on({
                self.client
                    .get(format!("http://{}:{}/health", self.host, self.port))
                    .send()
            });
        health.is_ok()
    }

    // specific callbacks (might want to handle this more extensibly...)

    pub async fn send_message(&self, messages: &Vec<Message>, sys_prompt: &str, use_tools: bool, model: Option<String>, randomness: f64) 
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        let params = SendMessageParamsBuilder::default()
            .messages(messages.to_vec())
            .sys_prompt(sys_prompt.to_string())
            .use_tools(use_tools)
            .model(model)
            .randomness(randomness)
            .build()?;
        let req = HttpRequestBuilder::default()
            .endpoint("send_message".into())
            .data(json!(params))
            .build()?;

        let res = self.post(
            format!("http://{}:{}/connect/chat/message", self.host, self.port), 
            json!(req)
        ).await?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(serde_json::from_value(res.data)?)
        }
    }

    pub fn get_tpm(&self) -> Result<(usize,usize), Box<dyn std::error::Error>> {
        let req = HttpRequestBuilder::default()
            .endpoint("get_tpm".into())
            .build()?;

        let res = tokio::runtime::Runtime::new()?
            .block_on(self.get(
                format!("http://{}:{}/connect/chat/get_tpm", self.host, self.port),
                json!(req)
        ))?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(serde_json::from_value(res.data)?)
        }
    }

    pub fn get_max_tpm(&self) -> Result<(usize,usize), Box<dyn std::error::Error>> {
        let req = HttpRequestBuilder::default()
            .endpoint("get_max_tpm".into())
            .build()?;

        let res = tokio::runtime::Runtime::new()?
            .block_on(self.get(
                format!("http://{}:{}/connect/chat/get_max_tpm", self.host, self.port),
                json!(req)
        ))?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(serde_json::from_value(res.data)?)
        }
    }

    pub fn list_tools(&self) -> Result<Vec<AnthropicToolDefinition>, Box<dyn std::error::Error>> {
        let req = HttpRequestBuilder::default()
            .endpoint("list_tools".into())
            .build()?;

        let res = tokio::runtime::Runtime::new()?
            .block_on(self.get(
                format!("http://{}:{}/tools/list_tools", self.host, self.port),
                json!(req)
        ))?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(serde_json::from_value(res.data)?)
        }
    }

    pub fn execute_tool(&self, name: &str, input: &Value) 
    -> Result<Vec<ToolResultContentBlock>, Box<dyn std::error::Error>> {
        let params = ExecuteToolParamsBuilder::default()
            .name(name.to_owned())
            .input(input.to_owned())
            .build()?;
        let req = HttpRequestBuilder::default()
            .endpoint("execute_tool".into())
            .data(json!(params))
            .build()?;

        let res = tokio::runtime::Runtime::new()?
            .block_on(self.post(
                format!("http://{}:{}/tools/execute_tool", self.host, self.port),
                json!(req)
        ))?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            let response: Vec<ToolResultContentBlock> = serde_json::from_value(res.data)?;
            Ok(response)
        }
    }

    pub async fn embed_files(&self, collection_name: &str, paths: Vec<&Path>) 
    -> Result<(), Box<dyn std::error::Error>> {
        let params = EmbedFilesParamsBuilder::default()
            .collection_name(collection_name.to_owned())
            .file_paths(paths.iter().map(|p| p.to_path_buf()).collect())
            .build()?;
        let req = HttpRequestBuilder::default()
            .endpoint("embed_files".into())
            .data(json!(params))
            .build()?;

        let res = self.post(
            format!("http://{}:{}/connect/embed_files", self.host, self.port),
            json!(req)
        ).await?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(())
        }
    }

    pub async fn query_vdb(&self, collection_name: &str, query: &str, search_limit: Option<u64>) 
    -> Result<Value, Box<dyn std::error::Error>> {
        let params = QueryVdbParamsBuilder::default()
            .collection_name(collection_name.to_string())
            .query(query.to_string())
            .search_limit(search_limit)
            .build()?;
        let req = HttpRequestBuilder::default()
            .endpoint("query_vdb".into())
            .data(json!(params))
            .build()?;

        // todo this should be a get (?)
        let res = self.post(
            format!("http://{}:{}/connect/query_vdb", self.host, self.port),
            json!(req)
        ).await?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(serde_json::from_value(res.data)?)
        }
    }

    pub async fn list_collections(&self, ) 
    -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let req = HttpRequestBuilder::default()
            .endpoint("list_collections".into())
            .build()?;

        let res = self.get(
            format!("http://{}:{}/connect/list_collections", self.host, self.port),
            json!(req)
        ).await?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(serde_json::from_value(res.data)?)
        }
    }

    pub async fn add_collection(&self, collection_name: &str)
    -> Result<(), Box<dyn std::error::Error>> {
        let req = HttpRequestBuilder::default()
            .endpoint("add_collection".into())
            .data(json!(collection_name))
            .build()?;

        let res = self.post(
            format!("http://{}:{}/connect/add_collection", self.host, self.port),
            json!(req)
        ).await?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(())
        }
    }

    pub fn set_collection(&self, collection_name: &str)
    -> Result<(), Box<dyn std::error::Error>> {
        let req = HttpRequestBuilder::default()
            .endpoint("set_collection".into())
            .data(json!(collection_name))
            .build()?;

        let res = tokio::runtime::Runtime::new()?
            .block_on(self.post(
                format!("http://{}:{}/state/set_collection", self.host, self.port),
                json!(req)
        ))?;
        let res = serde_json::from_value::<HttpResponse>(res)?;

        if res.status=="error" {
            Err("there was an error".into())
        } else {
            Ok(())
        }
    }
}

// signal(s)

async fn shutdown_signal() {
    signal::ctrl_c().await
        .expect("failed to install CTRL+C signal handler");
    log::info!("Shutdown signal received");
}

#[cfg(test)]
mod tests {

    use crate::connection::anthropic_client::{ContentBlock, Role};

    use super::*;

    use reqwest::{ClientBuilder};
    use serde_json::json;

    #[tokio::test]
    async fn test_run_server() {
        log::info!("Starting HTTP server");

        let conn_mgr = ConnectionManager::new();
        let tool_mgr = ToolManager::new();

        let mut server = HttpServer::new();
        server.init_router(Arc::new(conn_mgr), Arc::new(tool_mgr)).expect("Router init failed");
        server.init_tcp_listener().await.expect("TCP listener init failed");

        log::info!("Hosting backend on {}", server.addr());
        server.serve().await.expect("Server host failed");
    }

    #[tokio::test]
    async fn test_health() {
        let client = ClientBuilder::default().build().unwrap();

        let response = client
            .get("http://127.0.0.1:8081/health")
            .send().await.expect("failed to send");

        log::info!("Response: {response:?}");
    }

    #[tokio::test]
    async fn test_send_message() {
        let client = ClientBuilder::default().build().unwrap();

        let response = client
            .post("http://127.0.0.1:8081/connect/chat/message")
            .header("content-type", "application/json")
            .json(&json!({
                "endpoint": "send_message",
                "data": SendMessageParamsBuilder::default()
                    .messages(vec![
                        Message {
                            role: Role::User,
                            content: vec![
                                ContentBlock::Text { text: "hello, im justin".into() }
                            ]
                        }
                    ])
                    .build().unwrap()
            }))
            .send().await.expect("failed to send")
            .json::<serde_json::Value>().await.expect("failed to extract json");

        log::info!("Response: {response:?}");
    }

    #[tokio::test]
    async fn test_send_message_client() {
        let client = HttpClient::new();
        let response = client.send_message(
            &vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: "Hello im Justin!".into() }]
            }], 
            "This is a sys prompt yay", 
            false, 
            None,
            0.75
        ).await.expect("request failed");

        log::info!("Response: {response:?}");
    }
    
    #[tokio::test]
    async fn test_get_tpm() {
        let client = ClientBuilder::default().build().unwrap();

        let response = client
            .get("http://127.0.0.1:8081/connect/chat/get_tpm")
            .query(&json!({
                "endpoint": "get_tpm",
                "data": 0
            }))
            .send().await.expect("failed to send")
            .json::<serde_json::Value>().await.expect("failed to extract json");

        log::info!("Response: {response:?}");
    }

    #[tokio::test]
    async fn test_get_max_tpm() {
        let client = ClientBuilder::default().build().unwrap();

        let response = client
            .get("http://127.0.0.1:8081/connect/chat/get_max_tpm")
            .query(&json!({
                "endpoint": "get_max_tpm",
                "data": 0
            }))
            .send().await.expect("failed to send")
            .json::<serde_json::Value>().await.expect("failed to extract json");

        log::info!("Response: {response:?}");
    }
    
    #[tokio::test]
    async fn test_list_tools() {
        let client = ClientBuilder::default().build().unwrap();

        let response = client
            .get("http://127.0.0.1:8081/tools/list_tools")
            .query(&json!({
                "endpoint": "list_tools",
                "data": 0
            }))
            .send().await.expect("failed to send");
        let rslt = response
            .json::<serde_json::Value>().await.expect("failed to extract json");

        log::info!("Response: {rslt:?}");
    }
    
    #[tokio::test]
    async fn test_list_collections() {
        let client = ClientBuilder::default().build().unwrap();

        let response = client
            .get("http://127.0.0.1:8081/connect/list_collections")
            .query(&json!({
                "endpoint": "list_collections",
                "data": 0
            }))
            .send().await.expect("failed to send");
        let rslt = response
            .json::<serde_json::Value>().await.expect("failed to extract json");

        log::info!("Response: {rslt:?}");
    }
}