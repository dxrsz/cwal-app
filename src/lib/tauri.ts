// For defining the api exposed by tauri bindings.
export interface Player {
    alias: string;
    gateway: number;
}

export type ScrEvent = {
    ProfileSelect: Player;
} | {
    MatchFound: {
        player1: Player;
        player2: Player;
        map: string;
    };
} | {
    GameEnded: null;
} | {
    WebServerRunning: {
        port: number;
    };
} | {
    WebServerDown: null;
};