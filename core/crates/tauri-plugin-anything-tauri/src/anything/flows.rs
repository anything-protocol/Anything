use crate::{error::FlowResult, AnythingState, Error};
use anything_common::tracing;
use anything_graph::Flow;
use anything_persistence::{CreateFlowVersion, FlowVersion, UpdateFlow};
use serde::Serialize;

#[derive(Serialize)]
pub struct GetFlowsResponse {
    flows: Option<Vec<Flow>>,
}

#[tauri::command]
pub async fn get_flows(state: tauri::State<'_, AnythingState>) -> FlowResult<GetFlowsResponse> {
    match state.inner.try_lock() {
        Err(_e) => Err(Error::CoordinatorNotInitialized),
        Ok(ref inner) => match inner.get_flows().await {
            Ok(flows) => Ok(GetFlowsResponse { flows: Some(flows) }),
            Err(e) => {
                tracing::error!("Error getting flows: {:?}", e);
                Ok(GetFlowsResponse {
                    flows: Some(vec![]),
                })
            }
        },
    }
}

#[derive(Serialize)]
pub struct CreateFlowResponse {
    flow: Option<Flow>,
}

#[tauri::command]
pub async fn create_flow(
    state: tauri::State<'_, AnythingState>,
    flow_name: String,
) -> FlowResult<CreateFlowResponse> {
    tracing::debug!("Creating flow inside tauri plugin with name: {}", flow_name);
    // Acquire the lock on the Mutex
    match state.inner.try_lock() {
        Err(e) => {
            tracing::error!("Error getting lock on coordinator: {:?}", e);
            Err(Error::CoordinatorNotInitialized)
        }
        Ok(ref mut inner) => match inner.create_flow(flow_name).await {
            Ok(flow) => {
                tracing::debug!("Created flow inside tauri plugin");
                Ok(CreateFlowResponse { flow: Some(flow) })
            }
            Err(e) => {
                eprintln!("Error getting flows after creating flow: {:?}", e);
                Ok(CreateFlowResponse { flow: None })
            }
        },
    }
}

// #[derive(Serialize)]
// pub struct DeleteFlowResponse {
//     flow: Option<Flow>,
// }

// #[tauri::command]
// pub async fn delete_flow(
//     state: tauri::State<'_, AnythingState>,
//     flow_name: String,
// ) -> FlowResult<DeleteFlowResponse> {
//     match state.inner.try_lock() {
//         Err(_e) => Err(Error::CoordinatorNotInitialized),
//         Ok(ref inner) => match inner.delete_flow(flow_name).await {
//             Ok(flow) => Ok(DeleteFlowResponse { flow: Some(flow) }),
//             Err(e) => {
//                 eprintln!("Error getting flows: {:?}", e);
//                 Ok(DeleteFlowResponse { flow: None })
//             }
//         },
//     }
// }

#[derive(Serialize)]
pub struct UpdateFlowResponse {
    flow: Option<Flow>,
}

#[tauri::command]
pub async fn update_flow(
    state: tauri::State<'_, AnythingState>,
    flow_name: String,
    update_flow: UpdateFlow,
) -> FlowResult<UpdateFlowResponse> {
    match state.inner.try_lock() {
        Err(e) => {
            tracing::error!("Error getting lock on coordinator: {:?}", e);
            Err(Error::CoordinatorNotInitialized)
        }
        Ok(ref mut inner) => match inner.update_flow(flow_name, update_flow).await {
            Ok(flow) => {
                tracing::debug!("Created flow inside tauri plugin");
                Ok(UpdateFlowResponse { flow: Some(flow) })
            }
            Err(e) => {
                eprintln!("Error getting flows after creating flow: {:?}", e);
                Ok(UpdateFlowResponse { flow: None })
            }
        },
    }
}

#[derive(Serialize)]
pub struct CreateFlowVersionResponse {
    flow_version: Option<FlowVersion>,
}

#[tauri::command]
pub async fn create_flow_version(
    state: tauri::State<'_, AnythingState>,
    flow_name: String,
    create_flow: CreateFlowVersion,
) -> FlowResult<CreateFlowVersionResponse> {
    match state.inner.try_lock() {
        Err(e) => {
            tracing::error!("Error getting lock on coordinator: {:?}", e);
            Err(Error::CoordinatorNotInitialized)
        }
        Ok(ref mut inner) => match inner.create_flow_version(flow_name, create_flow).await {
            Ok(flow) => {
                tracing::debug!("Created flow inside tauri plugin");
                Ok(CreateFlowVersionResponse {
                    flow_version: Some(flow),
                })
            }
            Err(e) => {
                eprintln!("Error getting flows after creating flow: {:?}", e);
                Ok(CreateFlowVersionResponse { flow_version: None })
            }
        },
    }
}

#[derive(Serialize)]
pub struct ExecuteFlowResponse {}

#[tauri::command]
pub async fn execute_flow(
    state: tauri::State<'_, AnythingState>,
    flow_name: String,
) -> FlowResult<ExecuteFlowResponse> {
    match state.inner.try_lock() {
        Err(e) => {
            tracing::error!("Error getting lock on coordinator: {:?}", e);
            Err(Error::CoordinatorNotInitialized)
        }
        Ok(ref mut inner) => match inner.execute_flow(flow_name).await {
            Ok(_flow) => {
                tracing::debug!("Executed flow flow inside tauri plugin");
                Ok(ExecuteFlowResponse {})
            }
            Err(e) => {
                eprintln!("Error getting flows after executing flow: {:?}", e);
                Ok(ExecuteFlowResponse {})
            }
        },
    }
}
