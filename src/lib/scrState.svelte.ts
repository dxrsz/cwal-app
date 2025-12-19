import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Event } from "@tauri-apps/api/event";

export enum GameServerState {
  Running,
  NotRunning,
  Indeterminate, // haven't received tauri notification yet to say either way
}

export type ScrState = {
  gameServerState: GameServerState;
  port: number | null;
};

const scrState: ScrState = $state({
  gameServerState: GameServerState.Indeterminate,
  port: null,
  user: null,
  currentGame: null,
});

export const getScrState = () => scrState;

type BackendEvent =
  | {
      name: "WebServerRunning";
      payload: {
        port: number;
      };
    }
  | {
      name: "WebServerDown";
    };

// This function is used to convert the Rust event to a TypeScript event. The events are modeled
// in rust as an enum with a payload but idiomatic TypeScript would use a union type.
// For example, the Rust events:
//
// Event {
//     WebServerDown,
//     WebServerRunning { port: u16 },
// }
//
// should tranform into the BackendEvent typed above:
//
// {
//     name: 'WebServerRunning',
//     payload: { port: number }
// }
const convertBackendEvent = (ev: Event<object>): BackendEvent => {
  const name =
    typeof ev.payload === "string" ? ev.payload : Object.keys(ev.payload)[0];
  const payload =
    typeof ev.payload === "string" ? null : Object.values(ev.payload)[0];

  return {
    name,
    payload,
  } as BackendEvent;
};

export const configureReceiveBackendEvents = async () => {
  // Inform the backend to start generating events.
  await invoke("init_process");

  // Listen for them and modify our exposed state object
  return await listen("scr-event", (ev: Event<object>) => {
    const event = convertBackendEvent(ev);

    if ("WebServerDown" === event.name) {
      scrState.port = null;
      scrState.gameServerState = GameServerState.NotRunning;
    } else if ("WebServerRunning" === event.name) {
      scrState.port = event.payload.port;
      scrState.gameServerState = GameServerState.Running;
    }
  });
};
