import { SCApi, type BroodWarApiPath, type IBroodWarConnection } from 'bw-web-api';
import { getScrState } from '@/lib/scrState.svelte';
import { GravaticBooster, SCApiWithCaching } from 'gravatic-booster';
import {
    fetch as tauriFetch
} from '@tauri-apps/plugin-http';
import { LRUCache } from 'lru-cache';

export const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

export class TauriConnection implements IBroodWarConnection {
    constructor(private server: string) { }

    async fetch(path: BroodWarApiPath): Promise<string> {
        // LRU cache for stable endpoints; skip volatile ones below
        const normalizedPath = path.startsWith('/') ? path : `/${path}`;
        const shouldCache = !NO_CACHE_PREFIXES.some((p) => normalizedPath.startsWith(p));
        const key = `${this.server}${normalizedPath}`;
        if (shouldCache) {
            const cached = fetchCache.get(key);
            if (cached) {
                return cached;
            }
        }

        const max_attempts = 10;
        let timeout = 1000;
        let response: Response | null = null;

        for (let i = 0; i < max_attempts; i++) {
            await sleep(timeout * i);
            try {
                response = await tauriFetch(key, {
                    headers: {
                        Accept: "application/json",
                    },
                    connectTimeout: timeout * (i + 1)
                });
            } catch (error) {
                if (error instanceof DOMException && error.name === "TimeoutError") {
                    continue; // these are retriable
                }
                throw error;
            }

            const text = await response.text();
            const textLower = text.toLowerCase();

            if (textLower.startsWith("<!doctype") ||
                textLower.startsWith("internal error") ||
                textLower.startsWith("internal server error")
            ) {
                console.error("server error, retrying");
                continue; // these are retriable
            }

            // On success, cache if allowed (no TTL; evicts by LRU max)
            if (shouldCache) fetchCache.set(key, text);
            return text;
        }

        if (response) {
            throw Error(`Exceeded max retry attempt, status code: ${response.status} ${response.statusText}`);
        } else {
            throw Error("Exceeded max retry attempt, no response object.");
        }
    }
}

// Paths to avoid caching due to volatility
const NO_CACHE_PREFIXES = [
    '/web-api/v2/aurora-profile-by-toon',
    '/web-api/v1/leaderboard',
    '/web-api/v1/matchmaker-gameinfo-by-toon',
    '/web-api/v1/matchmaker-player-stat-by-toon',
];

// Module-level LRU cache with no TTL; items evicted by LRU when max is reached
const fetchCache = new LRUCache<string, string>({ max: 200 });

const createGB = async (port: number): Promise<GravaticBooster> =>
    await GravaticBooster.create(
        new SCApiWithCaching(
            new SCApi(
                new TauriConnection(`http://localhost:${port}`))));

const scrState = getScrState();

// Resolves when scrState.port is available
export const getGb = (): Promise<GravaticBooster> => {
    return new Promise((resolve) => {
        $effect(() => {
            if (scrState.port) {
                resolve(createGB(scrState.port));
            }
        });
    });
}