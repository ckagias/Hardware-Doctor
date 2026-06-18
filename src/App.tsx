import { useState } from "react";
import Sidebar from "./components/Sidebar";
import MicrophonePage from "./pages/MicrophonePage";
import HeadphonesPage from "./pages/HeadphonesPage";
import ComingSoonPage from "./pages/ComingSoonPage";
import { MODULES, ModuleId } from "./lib/modules";
import "./App.css";

function App() {
  const [active, setActive] = useState<ModuleId>("microphone");

  function renderPage() {
    switch (active) {
      case "microphone":
        return <MicrophonePage />;
      case "headphones":
        return <HeadphonesPage />;
      default: {
        const mod = MODULES.find((m) => m.id === active);
        return <ComingSoonPage label={mod?.label ?? "Coming Soon"} />;
      }
    }
  }

  return (
    <div className="app-shell">
      <Sidebar active={active} onSelect={setActive} />
      <main className="content">{renderPage()}</main>
    </div>
  );
}

export default App;
