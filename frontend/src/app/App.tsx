import { RouterProvider } from "react-router-dom";
import { router } from "./routes";
import { WebSocketProvider } from "../net/WebSocketProvider";

export default function App() {
  return (
    <WebSocketProvider>
      <RouterProvider router={router} />
    </WebSocketProvider>
  );
}