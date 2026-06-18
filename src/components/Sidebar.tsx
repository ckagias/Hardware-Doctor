import { MODULES, ModuleId } from "../lib/modules";

interface SidebarProps {
  active: ModuleId;
  onSelect: (id: ModuleId) => void;
}

export default function Sidebar({ active, onSelect }: SidebarProps) {
  return (
    <nav className="sidebar">
      <div className="sidebar-title">Trouble</div>
      <ul>
        {MODULES.map((mod) => (
          <li key={mod.id}>
            <button
              type="button"
              className={mod.id === active ? "nav-item active" : "nav-item"}
              onClick={() => onSelect(mod.id)}
            >
              <span>{mod.label}</span>
              {!mod.available && <span className="badge">soon</span>}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
