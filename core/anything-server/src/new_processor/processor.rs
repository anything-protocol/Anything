use crate::new_processor::execute_task::execute_task;
use crate::new_processor::flow_session_cache::FlowSessionData;
use crate::new_processor::parsing_utils::get_trigger_node;
use crate::workflow_types::{CreateTaskInput, WorkflowVersionDefinition};
use crate::AppState;
use chrono::Utc;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use uuid::Uuid;

use crate::new_processor::db_calls::{
    create_task, get_workflow_definition, update_flow_session_status, update_task_status,
};
use crate::task_types::{ActionType, FlowSessionStatus, Stage, TaskStatus, TriggerSessionStatus};

// Add this near your other type definitions
#[derive(Debug, Clone)]
pub struct ProcessorMessage {
    pub workflow_id: Uuid,
    pub version_id: Option<Uuid>, //When we are calling a workflow from a webhook, we don't have a version id
    pub flow_session_id: Uuid,
    pub trigger_task: Option<CreateTaskInput>,
}

pub async fn processor(
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("[PROCESSOR] Starting processor");

    // Create a shared set to track active flow sessions
    let active_flow_sessions = Arc::new(Mutex::new(HashSet::new()));
    // Get the receiver from the state
    let mut rx = state.processor_receiver.lock().await;
    // Guard againts too many workflows running at once
    let number_of_processors_semaphore = state.workflow_processor_semaphore.clone();

    while let Some(message) = rx.recv().await {
        let workflow_id = message.workflow_id;
        let version_id = message.version_id;
        let flow_session_id = message.flow_session_id;
        let trigger_task = message.trigger_task;

        println!("[PROCESSOR] Received workflow_id: {}", flow_session_id);

        // Check if this flow session is already being processed
        {
            // Use a scope block to automatically drop the lock when done
            let mut active_sessions = active_flow_sessions.lock().await;
            if !active_sessions.insert(flow_session_id) {
                debug!(
                    "[PROCESSOR] Flow session {} is already being processed, skipping",
                    flow_session_id
                );
                continue;
            }
            debug!(
                "[PROCESSOR] Added flow session {} to active sessions",
                flow_session_id
            );
            // Lock is automatically dropped here at end of scope
        }

        // Clone what we need for the new task
        let state = Arc::clone(&state);
        let permit = number_of_processors_semaphore
            .clone()
            .acquire_owned()
            .await
            .unwrap();
        let client = state.anything_client.clone();
        let active_flow_sessions = Arc::clone(&active_flow_sessions);

        // Spawn a new task for this workflow
        //SPAWN NEW PROCESSOR FOR EACH WORKFLOW
        tokio::spawn(async move {
            debug!(
                "[PROCESSOR] Starting workflow processing for {}",
                flow_session_id
            );

            let mut workflow_definition = None;

            // Try to get from cache first using a read lock
            {
                let cache = state.flow_session_cache.read().await;
                debug!(
                    "[PROCESSOR] Checking cache for flow_session_id: {}",
                    flow_session_id
                );
                if let Some(session_data) = cache.get(&flow_session_id) {
                    if let Some(workflow) = &session_data.workflow {
                        debug!(
                            "[PROCESSOR] Found workflow in cache for flow_session_id: {}",
                            flow_session_id
                        );
                        workflow_definition = Some(workflow.clone());
                    }
                }
            }

            //TODO: add the option for task hydration if a 3rd party system detects we need to pick up unfinisehd work
            //Most likely from deploying code or something with eratic shutdowns
            //Otherwise the service will be unusefull for possibly the longest we allow a run to go with carefull shutdown

            // Only fetch from DB if we didn't find it in cache
            if workflow_definition.is_none() {
                debug!(
                "[PROCESSOR] No workflow found in cache, fetching from DB for flow_session_id: {}",
                flow_session_id
            );

                let workflow =
                    match get_workflow_definition(state.clone(), &workflow_id, version_id.as_ref())
                        .await
                    {
                        Ok(w) => {
                            debug!("[PROCESSOR] Successfully fetched workflow from DB");
                            w
                        }
                        Err(e) => {
                            debug!("[PROCESSOR] Error getting workflow definition: {}", e);
                            return;
                        }
                    };

                // Only update cache if there isn't already data there
                {
                    let mut cache = state.flow_session_cache.write().await;
                    if cache.get(&flow_session_id).is_none() {
                        debug!("[PROCESSOR] Creating new session data in cache");
                        let session_data = FlowSessionData {
                            workflow: Some(workflow.clone()),
                            tasks: HashMap::new(),
                            flow_session_id,
                            workflow_id,
                            workflow_version_id: version_id,
                        };
                        cache.set(&flow_session_id, session_data);
                    }
                }

                workflow_definition = Some(workflow);
            }

            debug!(
                "[PROCESSOR] Workflow definition status: {:?}",
                workflow_definition.is_some()
            );

            //TODO: we need to find out how to deal with a workflow that say only gets half processed and we shut down.
            //How do we recover from that? -> Proper shutdown signal is probably needed.
            //WE ARE ASSUMING THAT THE WORKFLOW WILL BE COMPLETED IN THIS SINGLE PROCESSOR CALL.
            // ... existing code ...
            let workflow = match &workflow_definition {
                Some(w) => w,
                None => {
                    debug!("[PROCESSOR] No workflow definition found");
                    //This should never happen
                    return;
                }
            };

            debug!("[PROCESSOR] Starting workflow execution");

            // Create initial trigger task
            let trigger_node = get_trigger_node(&workflow.flow_definition).unwrap();

            let initial_task = if let Some(trigger_task) = trigger_task {
                trigger_task
            } else {
                CreateTaskInput {
                    account_id: workflow.account_id.to_string(),
                    processing_order: 0,
                    task_status: TaskStatus::Running.as_str().to_string(),
                    flow_id: workflow_id.to_string(),
                    flow_version_id: workflow.flow_version_id.to_string(),
                    action_label: trigger_node.label.clone(),
                    trigger_id: trigger_node.action_id.clone(),
                    trigger_session_id: Uuid::new_v4().to_string(),
                    trigger_session_status: TriggerSessionStatus::Running.as_str().to_string(),
                    flow_session_id: flow_session_id.to_string(),
                    flow_session_status: FlowSessionStatus::Running.as_str().to_string(),
                    action_id: trigger_node.action_id.clone(),
                    r#type: ActionType::Trigger,
                    plugin_id: trigger_node.plugin_id.clone(),
                    stage: if workflow.published {
                        Stage::Production.as_str().to_string()
                    } else {
                        Stage::Testing.as_str().to_string()
                    },
                    config: json!({
                        "variables": serde_json::json!(trigger_node.variables),
                        "input": serde_json::json!(trigger_node.input),
                    }),
                    result: None,
                    started_at: Some(Utc::now()),
                    test_config: None,
                }
            };

            // Start with trigger task
            let mut current_task = match create_task(state.clone(), &initial_task).await {
                Ok(task) => {
                    // Update cache with new task
                    let mut cache = state.flow_session_cache.write().await;
                    if cache.add_task(&flow_session_id, task.clone()) {
                        Some(task)
                    } else {
                        debug!(
                            "[PROCESSOR] Failed to add task to cache for flow_session_id: {}",
                            flow_session_id
                        );
                        Some(task)
                    }
                }
                Err(e) => {
                    debug!("[PROCESSOR] Error creating initial task: {}", e);
                    None
                }
            };

            // Create graph for BFS traversal
            let workflow_def: WorkflowVersionDefinition = workflow.flow_definition.clone();

            let mut graph: HashMap<String, Vec<String>> = HashMap::new();
            for edge in &workflow_def.edges {
                graph
                    .entry(edge.source.clone())
                    .or_insert_with(Vec::new)
                    .push(edge.target.clone());
            }

            let mut processing_order = 1;

            // Process tasks until workflow completion
            while let Some(task) = current_task {
                // Execute the current task and handle its result
                debug!("[PROCESSOR] Executing task: {}", task.task_id);
                let task_result = match execute_task(state.clone(), &client, &task).await {
                    Ok(success_value) => {
                        debug!("[PROCESSOR] Task {} completed successfully", task.task_id);
                        Ok(success_value)
                    }
                    Err(error) => {
                        debug!("[PROCESSOR] Task {} failed: {:?}", task.task_id, error);

                        // Update task status to failed
                        let state_clone = state.clone();
                        let task_id = task.task_id.clone();
                        let error_clone = error.clone();
                        tokio::spawn(async move {
                            if let Err(e) = update_task_status(
                                state_clone,
                                &task_id,
                                &TaskStatus::Failed,
                                Some(error_clone),
                            )
                            .await
                            {
                                debug!("[PROCESSOR] Failed to update task status: {}", e);
                            }
                        });

                        // Update flow session status to failed
                        let state_clone = state.clone();
                        let flow_session_id_clone = flow_session_id.clone();
                        tokio::spawn(async move {
                            if let Err(e) = update_flow_session_status(
                                &state_clone,
                                &flow_session_id_clone,
                                &FlowSessionStatus::Failed,
                                &TriggerSessionStatus::Failed,
                            )
                            .await
                            {
                                debug!("[PROCESSOR] Failed to update flow session status: {}", e);
                            }
                        });

                        // Update cache
                        {
                            let mut cache = state.flow_session_cache.write().await;
                            let mut task_copy = task.clone();
                            task_copy.result = Some(error.clone());
                            task_copy.task_status = TaskStatus::Failed;
                            task_copy.ended_at = Some(Utc::now());
                            let _ = cache.update_task(&flow_session_id, task_copy);
                        }

                        debug!("[PROCESSOR] Workflow failed: {}", flow_session_id);
                        // current_task = None; // Exit the processing loop
                        break; // Exit the while loop
                    }
                };

                // Always wrap the result in Some() for storage, regardless of success/failure
                let result_for_storage = Some(match task_result {
                    Ok(value) => value,
                    Err(err) => err,
                });

                // Spawn task status update to DB asynchronously
                let state_clone = state.clone();
                let task_id = task.task_id.clone();
                let result_clone = result_for_storage.clone();
                tokio::spawn(async move {
                    if let Err(e) = update_task_status(
                        state_clone,
                        &task_id,
                        &TaskStatus::Completed,
                        result_clone,
                    )
                    .await
                    {
                        debug!("[PROCESSOR] Failed to update task status: {}", e);
                    }
                });

                //Update cache with result the same we do the db. these need to match!
                {
                    let mut cache = state.flow_session_cache.write().await;
                    let mut task_copy = task.clone();
                    task_copy.result = result_for_storage;
                    task_copy.task_status = TaskStatus::Completed;
                    task_copy.ended_at = Some(Utc::now());
                    let _ = cache.update_task(&flow_session_id, task_copy);
                }

                let next_action = if let Some(neighbors) = graph.get(&task.action_id) {
                    let mut next_action = None;
                    // Get the first unprocessed neighbor
                    //TODO: this is where we would handle if we have multiple paths to take and can parallelize
                    for neighbor_id in neighbors {
                        let neighbor = workflow_def
                            .actions
                            .iter()
                            .find(|action| &action.action_id == neighbor_id);

                        if let Some(action) = neighbor {
                            // Check if this task has already been processed
                            let cache = state.flow_session_cache.read().await;
                            if let Some(session_data) = cache.get(&flow_session_id) {
                                if !session_data
                                    .tasks
                                    .iter()
                                    .any(|(_, t)| t.action_id == action.action_id)
                                {
                                    next_action = Some(action.clone());
                                    break;
                                }
                            }
                        }
                    }
                    next_action
                } else {
                    None
                };

                // Create next task if available
                current_task = if let Some(next_action) = next_action {
                    let next_task_input = CreateTaskInput {
                        account_id: workflow.account_id.to_string(),
                        processing_order,
                        task_status: TaskStatus::Running.as_str().to_string(), //we create tasks when we start them
                        flow_id: workflow_id.to_string(),
                        flow_version_id: workflow.flow_version_id.to_string(),
                        action_label: next_action.label.clone(),
                        trigger_id: next_action.action_id.clone(),
                        trigger_session_id: Uuid::new_v4().to_string(),
                        trigger_session_status: TriggerSessionStatus::Pending.as_str().to_string(),
                        flow_session_id: flow_session_id.to_string(),
                        flow_session_status: FlowSessionStatus::Pending.as_str().to_string(),
                        action_id: next_action.action_id,
                        r#type: next_action.r#type,
                        plugin_id: next_action.plugin_id.clone(),
                        stage: if workflow.published {
                            Stage::Production.as_str().to_string()
                        } else {
                            Stage::Testing.as_str().to_string()
                        },
                        config: json!({
                            "variables": serde_json::json!(next_action.variables),
                            "input": serde_json::json!(next_action.input),
                        }),
                        result: None,
                        test_config: None,
                        started_at: Some(Utc::now()),
                    };

                    match create_task(state.clone(), &next_task_input).await {
                        Ok(new_task) => {
                            // Update cache
                            {
                                let mut cache = state.flow_session_cache.write().await;
                                if let Some(mut session_data) = cache.get(&flow_session_id) {
                                    session_data
                                        .tasks
                                        .insert(new_task.task_id.clone(), new_task.clone());
                                    cache.set(&flow_session_id, session_data);
                                }
                            } // Lock is dropped here
                            processing_order += 1;
                            Some(new_task)
                        }
                        Err(e) => {
                            debug!("[PROCESSOR] Error creating next task: {}", e);
                            None
                        }
                    }
                } else {
                    // No more tasks - workflow is complete
                    let state_clone = state.clone();
                    let flow_session_id_clone = flow_session_id.clone();
                    tokio::spawn(async move {
                        if let Err(e) = update_flow_session_status(
                            &state_clone,
                            &flow_session_id_clone,
                            &FlowSessionStatus::Completed,
                            &TriggerSessionStatus::Completed,
                        )
                        .await
                        {
                            debug!("[PROCESSOR] Failed to update flow session status: {}", e);
                        }
                    });

                    debug!("[PROCESSOR] Workflow completed: {}", flow_session_id);
                    None
                };
            }

            debug!(
                "[PROCESSOR] Completed workflow processing for {}",
                flow_session_id
            );

            // Invalidate cache for completed flow session
            {
                let mut cache = state.flow_session_cache.write().await;
                cache.invalidate(&flow_session_id);
                debug!(
                    "[PROCESSOR] Removed flow session {} from cache",
                    flow_session_id
                );
            }
            //TODO: error handling
            //TODO: handle updating flow session and trigger session status et

            // // Remove the flow session from active sessions when done
            active_flow_sessions.lock().await.remove(&flow_session_id);
            drop(permit);
        });
        //END SPAWNED PROCESSOR
    }

    Ok(())
}

//TODO:
//Traverse the worfklow definition to get next task
//Update task status in cache and db
//Bundle the task
//Run task
//Update status and result in cache and db
//Determine if workflow is complete
//If complete, update flow session status in cache and db
//If not complete, update flow session with next task in line
//Send signal to webhook engine if response is needed
