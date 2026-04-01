use anyhow::Result;
use dashmap::DashMap;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
    transport::io::stdio,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Task {
    #[schemars(description = "the task id")]
    id: String,
    #[schemars(description = "the task name")]
    name: String,
}

#[derive(Debug, Clone)]
pub struct TODO {
    tasks: DashMap<String, Task>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AddTaskRequest {
    #[schemars(description = "the task name")]
    pub name: String,
}
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteTaskRequest {
    #[schemars(description = "the task id")]
    pub id: String,
}

#[tool_router]
impl TODO {
    fn new() -> Self {
        TODO {
            tool_router: Self::tool_router(),
            tasks: DashMap::new(),
        }
    }

    #[tool(description = "add new task")]
    async fn add(
        &self,
        Parameters(AddTaskRequest { name }): Parameters<AddTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        let id = Uuid::now_v7().to_string();
        let task = Task {
            id: id.clone(),
            name,
        };
        self.tasks.insert(id.clone(), task.clone());
        Ok(CallToolResult::structured(json!(task)))
    }

    #[tool(description = "get all tasks")]
    async fn all(&self) -> Result<CallToolResult, McpError> {
        let tasks: Vec<Task> = self
            .tasks
            .clone()
            .into_iter()
            .map(|(_, task)| task)
            .collect();
        Ok(CallToolResult::structured(json!(tasks)))
    }

    #[tool(description = "delete task by id")]
    async fn delete(&self, Parameters(DeleteTaskRequest { id }): Parameters<DeleteTaskRequest>) {
        self.tasks.remove(&id);
    }

    #[tool(description = "delete all tasks")]
    async fn delete_all(&self) {
        self.tasks.clear();
    }
}

#[tool_handler]
impl ServerHandler for TODO {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("A simple TODO list".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    let transport = stdio();
    let service = TODO::new().serve(transport).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;
    service.waiting().await?;
    Ok(())
}
