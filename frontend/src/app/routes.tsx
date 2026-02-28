import { createBrowserRouter } from "react-router-dom";
import { HomePage } from "../pages/HomePage";
import { LobbyPage } from "../pages/LobbyPage";
import { GamePage } from "../pages/GamePage";

export const router = createBrowserRouter([
  { path: "/", element: <HomePage /> },
  { path: "/lobby", element: <LobbyPage /> },
  { path: "/game/:gameId", element: <GamePage /> },
]);