use rmcp::{RoleServer, ServerHandler, service::RequestContext, model::{CallToolRequestParams, CallToolResult, InitializeRequestParams, InitializeResult, ListToolsResult, PaginatedRequestParams}, ErrorData};

#[allow(dead_code)]
struct MyHandler;

impl ServerHandler for MyHandler {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::LATEST,
            capabilities: rmcp::model::ServerCapabilities::default(),
            server_info: rmcp::model::Implementation {
                name: "check-deps".into(),
                version: "0.1.0".into(),
                ..rmcp::model::Implementation::default()
            },
            instructions: None,
        }
    }

    async fn initialize(
        &self,
        _params: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        Ok(InitializeResult::default())
    }

    async fn list_tools(
        &self,
        _req: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::default())
    }

    async fn call_tool(
        &self,
        _req: CallToolRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(CallToolResult::success(vec![]))
    }
}

fn main() {}
