import React from "react";
import ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import Layout from "./layout";
import ErrorPage from "./error-page";
// Routes
import Home from "./routes/home";
import Settings from "./routes/settings";
import Tables from "./routes/tables";
import FlowEditor from "./routes/flowEditor";
import TableData from "./routes/tableData";
import Flows from "./routes/flows";
import Models from "./routes/models";
import Vectors from "./routes/vectors";
import Chats from "./routes/chats";
import ChatInterface from "./routes/chatInterface";
// Contexts
import { TauriProvider } from "./context/TauriProvider";
import { SettingsProvider } from "./context/SettingsProvider";
import { LocalFileProvider } from "./context/LocalFileProvider";
import { SqlProvider } from "./context/SqlProvider";
import { ModelProvider } from "./context/ModelsProvider";
import "./styles.css";
import Templates from "./routes/templates";

const VITE_PUBLIC_POSTHOG_KEY = import.meta.env.VITE_PUBLIC_POSTHOG_KEY;
const VITE_PUBLIC_POSTHOG_HOST = import.meta.env.VITE_PUBLIC_POSTHOG_HOST;

import { PostHogProvider } from "posthog-js/react";

let posthog;

if (
  import.meta.env.mode === "production"
) {
  console.log("Initializing PostHog in production");
  posthog.init(process.env.VITE_PUBLIC_POSTHOG_KEY, {
    api_host: process.env.VITE_PUBLIC_POSTHOG_HOST,
  });
} else {
  console.log("Initializing PostHog in development");
  console.log("import.meta.env", import.meta.env);
}

const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    errorElement: <ErrorPage />,
    children: [
      {
        index: true,
        element: <Home />,
      },
      {
        path: "/flows",
        element: <Flows />,
      },
      {
        path: "/templates",
        element: <Templates />,
      },
      {
        path: "/templates/:author_name/:template_name",
        element: <Templates />,
      },
      {
        path: "/models",
        element: <Models />,
      },
      {
        path: "/vectors",
        element: <Vectors />,
      },
      {
        path: "flows/:flow_name",
        element: <FlowEditor />,
      },
      {
        path: "/chats",
        element: <Chats />,
      },
      {
        path: "/chats/:flow_id",
        element: <ChatInterface />,
      },
      {
        path: "/tables",
        element: <Tables />,
      },
      {
        path: "/tables/:table",
        element: <TableData />,
      },
      {
        path: "/settings",
        element: <Settings />,
      },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <PostHogProvider client={posthog}>
      <TauriProvider>
        <LocalFileProvider>
          <ModelProvider>
            <SqlProvider>
              <SettingsProvider>
                <RouterProvider router={router} />
              </SettingsProvider>
            </SqlProvider>
          </ModelProvider>
        </LocalFileProvider>
      </TauriProvider>
    </PostHogProvider>
  </React.StrictMode>
);
