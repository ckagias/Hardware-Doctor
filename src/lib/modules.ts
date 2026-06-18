export type ModuleId = "microphone" | "headphones" | "keyboard" | "mouse" | "controller";

export interface DeviceModule {
  id: ModuleId;
  label: string;
  description: string;
  available: boolean;
}

export const MODULES: DeviceModule[] = [
  {
    id: "microphone",
    label: "Microphone",
    description: "Test input level and listen back to your mic",
    available: true,
  },
  {
    id: "headphones",
    label: "Headphones / Speakers",
    description: "Test left/right channels and playback",
    available: true,
  },
  {
    id: "keyboard",
    label: "Keyboard",
    description: "Coming soon",
    available: false,
  },
  {
    id: "mouse",
    label: "Mouse",
    description: "Coming soon",
    available: false,
  },
  {
    id: "controller",
    label: "Controller",
    description: "Coming soon",
    available: false,
  },
];
