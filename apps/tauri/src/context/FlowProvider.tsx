import { parse } from "iarna-toml-esm";
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { useParams } from "react-router-dom";
import {
  addEdge,
  applyEdgeChanges,
  applyNodeChanges,
  Connection,
  Edge,
  EdgeChange,
  Node,
  NodeChange,
  OnConnect,
  OnEdgesChange,
  OnNodesChange,
  ReactFlowInstance,
} from "reactflow";
import { FlowFrontMatter } from "../utils/flowTypes";
import { ProcessingStatus, SessionComplete } from "../utils/eventTypes";

import api from "../tauri_api/api";
import { useFlowsContext } from "./FlowsProvider";
import { Node as FlowNode } from "../utils/flowTypes"; 

function findNextNodeId(nodes: any): string {
  // Return 1 if there are no nodes
  if (!nodes) {
    console.log("no nodes in FindNextNodeId, returning id 1");
    return "1";
  }
  // Initialize the maxId to 0
  let maxId = 0;

  // Loop through the nodes and find the maximum numeric ID value
  nodes.forEach((node: any) => {
    const numericId = parseInt(node.id, 10);
    if (!isNaN(numericId) && numericId > maxId) {
      maxId = numericId;
    }
  });
  // Increment the maxId to get the next ID for the new node
  const nextId = (maxId + 1).toString();

  return nextId;
}

interface FlowContextInterface {
  nodes: Node[];
  edges: Edge[];
  flowFrontmatter: FlowFrontMatter | undefined;
  currentProcessingStatus: ProcessingStatus | undefined;
  currentProcessingSessionId: string | undefined;
  onNodesChange: OnNodesChange;
  onEdgesChange: OnEdgesChange;
  onConnect: OnConnect;
  toml: string;
  onDragOver: (event: any) => void;
  onDrop: (event: any, reactFlowWrapper: any) => void;
  addNode: (type: string, specialData?: any) => void;
  setReactFlowInstance: (instance: ReactFlowInstance | null) => void;
  updateFlowFrontmatter: (flow_name: string, keysToUpdate: any) => void;
}

export const FlowContext = createContext<FlowContextInterface>({
  toml: "",
  nodes: [],
  edges: [],
  flowFrontmatter: undefined,
  currentProcessingStatus: undefined,
  currentProcessingSessionId: undefined,
  onNodesChange: () => {},
  onEdgesChange: () => {},
  onConnect: () => {},
  onDragOver: () => {},
  onDrop: () => {},
  addNode: () => {},
  setReactFlowInstance: () => {},
  updateFlowFrontmatter: () => {},
});

export const useFlowContext = () => useContext(FlowContext);

export const FlowProvider = ({ children }: { children: ReactNode }) => {
  const { updateFlow } = useFlowsContext();
  const { flow_name } = useParams();
  const [initialTomlLoaded, setInitialTomlLoaded] = useState<boolean>(false);
  const [loadingToml, setLoadingToml] = useState<boolean>(false);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [flowFrontmatter, setFlowFrontmatter] = useState<
    FlowFrontMatter | undefined
  >();
  const [toml, setToml] = useState<string>("");
  // State for managing current processing for manual triggers and ebugging
  const [currentProcessingStatus, setCurrentProcessingStatus] = useState<
    ProcessingStatus | undefined
  >();
  const [currentProcessingSessionId, setCurrentProcessingSessionId] = useState<
    string | undefined
  >();
  const [sessionComplete, setSessionComplete] = useState<
    SessionComplete | undefined
  >();
  const [reactFlowInstance, setReactFlowInstance] =
    useState<ReactFlowInstance | null>(null);
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const addNode = (
    position: { x: number; y: number },
    specialData?: any
  ) => {
    const nextId = findNextNodeId(nodes);
    const newNode: Node = {
      id: nextId,
      type: "superNode",
      position,
      data: { ...specialData },
    };

    setNodes((nodes) => {
      return [...nodes, newNode];
    });
  };

  const onNodesChange: OnNodesChange = (nodeChanges: NodeChange[]) => {
    console.log("onNodesChange nodeChanges", nodeChanges);
    setNodes((nodes) => {
      return applyNodeChanges(nodeChanges, nodes);
    });
  };

  const onEdgesChange: OnEdgesChange = (edgeChanges: EdgeChange[]) => {
    setEdges((edges) => {
      return applyEdgeChanges(edgeChanges, edges);
    });
  };

  const onConnect: OnConnect = (params: Connection) => {
    setEdges((edges) => {
      return addEdge(params, edges);
    });
  };

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault();
    if (event.dataTransfer === null) return;
    event.dataTransfer.dropEffect = "move";
  }, []);

  const onDrop = useCallback(
    (event: DragEvent, reactFlowWrapper: any) => {
      event.preventDefault();
      const reactFlowBounds = reactFlowWrapper.current.getBoundingClientRect();
      if (event.dataTransfer === null) return;

      const nodeData: FlowNode = JSON.parse(event.dataTransfer.getData("nodeData"));

      if (typeof nodeData === "undefined" || !nodeData) {
        return;
      }
  
      if (!reactFlowInstance) throw new Error("reactFlowInstance is undefined");

      let position = reactFlowInstance.project({
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      });

      addNode(position, nodeData);
    },
    [addNode]
  );

  const updateFlowFrontmatter = async (
    flow_name: string,
    keysToUpdate: any
  ) => {
    try {
      // if we are updating name in TOML we also need to update the folder name
      if (keysToUpdate.name) {
        await updateFlow(flow_name, keysToUpdate.name);
      }
      let flow_frontmatter = { ...flowFrontmatter, ...keysToUpdate };
      setFlowFrontmatter(flow_frontmatter);
    } catch (error) {
      console.log("error updating flow frontmatter", error);
    }
  };

  const writeToml = async (flow_id: string, toml: string) => {
    try {
      await api.flows.writeToml(flow_id, toml);
    } catch (error) {
      console.log("error saving toml", error);
    }
  };

  const readToml = async () => {
    try {
      //TODO:
      //RUST_MIGRATION
      // if (!flow_name) {
      //   throw new Error("appDocuments or flow_name is undefined");
      // }
      // console.log("reading toml in FlowProvider");
      // return await api.fs.readTextFile(
      //   appDocuments + "/flows/" + flow_name + "/flow.toml"
      // );

      return "";
    } catch (error) {
      console.log("error reading toml in FlowProvider", error);
      return "";
    }
  };

  //we have heard there is new toml
  const updateStateFromToml = async () => {
    try {
      let new_toml = await readToml();
      if (!new_toml) throw new Error("new_toml is undefined");
      //don't update if nothing has changed in toml file
      if (new_toml === toml) return;
      setToml(new_toml);
      let parsedToml = parse(new_toml);

      if (!parsedToml.nodes) {
        parsedToml.nodes = [];
      }
      setNodes(parsedToml.nodes as any);
      if (!parsedToml.edges) {
        parsedToml.edges = [];
      }

      setNodes(parsedToml.nodes as any);
      setEdges(parsedToml.edges as any);
      setFlowFrontmatter(parsedToml.flow as FlowFrontMatter);
    } catch (error) {
      console.log("error loading toml in FlowProvider", error);
    }
  };

  // useEffect(() => {
  //   const fetchData = async () => {
  //     if (flow_name && !initialTomlLoaded && !loadingToml) {
  //       console.log("hydrating initial TOML");
  //       setLoadingToml(true);
  //       await updateStateFromToml();
  //       setInitialTomlLoaded(true);
  //       setLoadingToml(false);
  //     }
  //   };

  //   fetchData();
  // }, [flow_name, initialTomlLoaded]);

  const fetchFlow = async () => {
    try {
      console.log("Fetch Flow By Name", flow_name);
      if (!flow_name) return;
      let { flow } = await api.flows.getFlowByName<GetFlowResponse>(flow_name);
      console.log(
        "FLow Result in flow provider",
        JSON.stringify(flow, null, 3)
      );

      setFlowFrontmatter(flow);

      // let flow_versions = await api.getFlowVersions(flow.flow_id);

      // console.log(
      //   "Flow versions response",
      //   JSON.stringify(flow_versions, null, 3)
      // );

      let flow_versions = [
        {
          id: "a3893cf7-4683-40cd-9b42-b3de6e32e7e0",
          version: "0.0.1",
          description: "",
        },
      ];
    } catch (e) {
      console.log("error in fetch flow", JSON.stringify(e, null, 3));
    }
  };

  //Debounced write state to toml used for when we draggin things around.
  // useEffect(() => {
  //   // Clear any existing timers
  //   if (timerRef.current) {
  //     clearTimeout(timerRef.current);
  //   }

  //   // Set a new timer to write to TOML file
  //   timerRef.current = setTimeout(async () => {
  //     if (!initialTomlLoaded || loadingToml) return;

  //     let newToml = stringify({
  //       flow: flowFrontmatter as FlowFrontMatter,
  //       nodes: nodes as any,
  //       edges: edges as any,
  //     });
  //     console.log("writing to toml");
  //     // console.log(newToml);
  //     //don't write if nothing has changed in react state
  //     if (newToml === toml) return;
  //     setToml(newToml);
  //     await writeToml(newToml);
  //   }, 200);

  //   // Clean up
  //   return () => {
  //     if (timerRef.current) {
  //       clearTimeout(timerRef.current);
  //     }
  //   };
  // }, [nodes, edges, flowFrontmatter]);

  //Watch TOML file for changes
  // useEffect(() => {
  //   if (!initialTomlLoaded) return;
  //   let stopWatching = () => {};
  //   let path = `${appDocuments}/flows/${flow_name}/flow.toml`;

  //   console.log(`Watching ${path} for changes`);

  //   const watchThisFile = async () => {
  //     stopWatching = await api.watch.watchImmediate(path, (event) => {
  //       console.log("TOML file changed");
  //       updateStateFromToml();
  //     });
  //   };

  //   watchThisFile();
  //   return () => {
  //     stopWatching();
  //   };
  // }, [initialTomlLoaded]);

  //Watch event processing for fun ui updates
  useEffect(() => {
    let unlisten = api.subscribeToEvent("event_processing", (event: any) => {
      setCurrentProcessingStatus(event);
    });
    let unlisten2 = api.subscribeToEvent("session_complete", (event: any) => {
      setSessionComplete(event);
    });

    return () => {
      unlisten.then((unlisten) => unlisten());
      unlisten2.then((unlisten) => unlisten());
    };
  }, [currentProcessingSessionId]);

  //Hydrate all flow data on navigation
  //User params fetches url params from React-Router-Dom
  useEffect(() => {
    if (!flow_name) return;
    fetchFlow();
  }, [flow_name]);

  return (
    <FlowContext.Provider
      value={{
        nodes,
        edges,
        flowFrontmatter,
        currentProcessingStatus,
        currentProcessingSessionId,
        onConnect,
        onNodesChange,
        onEdgesChange,
        onDragOver,
        onDrop,
        toml,
        addNode,
        setReactFlowInstance,
        updateFlowFrontmatter,
      }}
    >
      {children}
    </FlowContext.Provider>
  );
};
