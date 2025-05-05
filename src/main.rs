use anyhow::Result;
use dashmap::DashMap;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::wrapper::Json,
    model::{ServerCapabilities, ServerInfo},
    tool,
    transport::io::stdio,
};
use serde::Serialize;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct TODO {
    tasks: DashMap<String, Task>,
}

#[tool(tool_box)]
impl TODO {
    fn new() -> Self {
        TODO {
            tasks: DashMap::new(),
        }
    }

    #[tool(description = "add new task")]
    async fn add(
        &self,
        #[tool(param)]
        #[schemars(description = "the task name")]
        name: String,
    ) -> Json<Task> {
        let id = Uuid::new_v4().to_string();
        let task = Task {
            id: id.clone(),
            name,
        };
        self.tasks.insert(id.clone(), task.clone());
        Json(task)
    }

    #[tool(description = "get all tasks")]
    async fn all(&self) -> Json<Vec<Task>> {
        let tasks: Vec<Task> = self
            .tasks
            .clone()
            .into_iter()
            .map(|(_, task)| task)
            .collect();
        Json(tasks)
    }

    #[tool(description = "delete task by id")]
    async fn delete(
        &self,
        #[tool(param)]
        #[schemars(description = "the task id")]
        id: String,
    ) {
        self.tasks.remove(&id);
    }

    #[tool(description = "delete all tasks")]
    async fn delete_all(&self) {
        self.tasks.clear();
    }
}

#[tool(tool_box)]
impl ServerHandler for TODO {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple TODO list".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
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
