<script lang="ts">
  import { onMount, untrack } from "svelte";

  import { invoke } from "@tauri-apps/api/core";
  import type { Match } from "gravatic-booster";
  import TimeAgo from "javascript-time-ago";
  import { toast } from "svelte-sonner";

  import MapName from "@/lib/components/MapName.svelte";
  import Race from "@/lib/components/icons/race.svelte";
  import { Button } from "@/lib/components/ui/button";
  import { Skeleton } from "@/lib/components/ui/skeleton";
  import * as Tooltip from "@/lib/components/ui/tooltip";
  import { getLimitsStore } from "@/lib/limits.svelte";
  import { getSettingsStore } from "@/lib/settingsStore.svelte";

  let limits = getLimitsStore();

  interface ChatMessage {
    timestamp: number; // ms since game start
    player: string;
    player_id: number;
    message: string;
  }

  interface ReplayDataMinimal {
    parsed_data: {
      game_duration_ms: number;
      chat_messages: ChatMessage[];
    };
    timestamp: string;
  }

  interface Props {
    match: Match;
    replayData?: ReplayDataMinimal;
    onOpenChat: (messages: ChatMessage[]) => void;
    onSetReplayData?: (data: ReplayDataMinimal) => void;
    isBlurred?: boolean;
  }

  const {
    match,
    replayData,
    onOpenChat,
    onSetReplayData,
    isBlurred = false,
  }: Props = $props();

  let internalReplayData = $state<ReplayDataMinimal | undefined>(
    untrack(() => replayData),
  );
  let loading = $state(false);
  const settingsStorePromise = getSettingsStore();
  let settingsStore: Awaited<ReturnType<typeof getSettingsStore>> | null = null;
  let loadSettings = async () => {
    if (!settingsStore) settingsStore = await settingsStorePromise;
    return settingsStore;
  };
  // Track external replayData prop updates (when parent cache fills after row mount)
  $effect(() => {
    if (!internalReplayData && replayData) {
      internalReplayData = replayData;
    }
  });
  let isDownloading = $state(false);

  let timeAgo = $state<TimeAgo | null>(null);

  const formatDuration = (durationMs: number): string => {
    const totalSeconds = Math.floor(durationMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}m${seconds.toString().padStart(2, "0")}s`;
  };

  const getMatchResult = (player: any) => {
    if (
      player?.profileInfo?.points?.delta !== undefined &&
      player.profileInfo.points.delta !== 0
    ) {
      return player.profileInfo.points.delta > 0 ? "win" : "loss";
    }
    return player?.result || "unknown";
  };

  const downloadReplay = async () => {
    await loadSettings();
    if (!settingsStore) return;
    if (isDownloading) return;
    isDownloading = true;
    try {
      const replays = await match.replays;
      const replay = replays.anyReplay;

      if (!replay) {
        toast.error("No replay available for this match");
        return;
      }

      limits.numReplayDownloads++;

      const sanitizeFilename = (filename: string) =>
        filename.replace(/[<>:"/\\|?*]/g, "_");

      const getRaceInitial = (race?: string) =>
        race && race.length > 0 ? race[0].toUpperCase() : "U";

      const generateReplayFilename = () => {
        const ts =
          (match.timestamp || replay.timestamp)?.toISOString() || "unknown";
        const p1Alias = match.thisPlayer?.toon || "Unknown";
        const p1Race = getRaceInitial(match.thisPlayer?.race);
        const p2Alias = match.opponent?.toon || "Unknown";
        const p2Race = getRaceInitial(match.opponent?.race);
        return sanitizeFilename(
          `${ts}_${p1Alias}(${p1Race})_vs_${p2Alias}(${p2Race}).rep`,
        );
      };

      const replayDownloadName = generateReplayFilename();

      const result = await invoke<string>("download_file", {
        url: replay.url,
        destinationPath: settingsStore.settings.replayDownloadPath,
        filename: replayDownloadName,
      });

      toast.success("Replay downloaded", {
        action: {
          label: "Open",
          onClick: () => {
            invoke("reveal_in_folder", { path: result }).catch((e) =>
              console.error("Failed to reveal file", e),
            );
          },
        },
      });
    } catch (error) {
      toast.error("Download failed", { description: String(error) });
    } finally {
      isDownloading = false;
    }
  };

  let lastParseError: string | null = null;
  const maybeParseReplay = async () => {
    if (internalReplayData || loading) return;
    loading = true;
    lastParseError = null;
    try {
      const replays = await match.replays;
      const replay = replays.anyReplay;
      if (!replay) return;
      await loadSettings();
      const destinationPath = settingsStore
        ? settingsStore.settings.replayDownloadPath
        : ""; // fallback

      const sanitizeFilename = (filename: string) =>
        filename.replace(/[<>:"/\\|?*]/g, "_");
      const getRaceInitial = (race?: string) =>
        race && race.length > 0 ? race[0].toUpperCase() : "U";
      const generateReplayFilename = () => {
        const ts =
          (match.timestamp || replay.timestamp)?.toISOString() || "unknown";
        const p1Alias = match.thisPlayer?.toon || "Unknown";
        const p1Race = getRaceInitial(match.thisPlayer?.race);
        const p2Alias = match.opponent?.toon || "Unknown";
        const p2Race = getRaceInitial(match.opponent?.race);
        return sanitizeFilename(
          `${ts}_${p1Alias}(${p1Race})_vs_${p2Alias}(${p2Race})_auto.rep`,
        );
      };

      interface DownloadAndParseReplayResponse {
        duration_ms: number;
        cached: boolean;
        start_time_ms: number;
        chat_messages: Array<{
          sender_name: string;
          message: string;
          frame_number: number;
          sender_id: number;
          timestamp_ms: number;
        }>;
      }

      const filename = generateReplayFilename();
      const parsed: DownloadAndParseReplayResponse = await invoke(
        "download_and_parse_replay",
        {
          url: replay.url,
          destinationPath,
          filename,
        },
      );

      if (!parsed.cached) {
        limits.numReplayDownloads++;
      }

      const mapped: ReplayDataMinimal = {
        parsed_data: {
          game_duration_ms: parsed.duration_ms,
          chat_messages: parsed.chat_messages.map((m) => ({
            timestamp: m.timestamp_ms,
            player: m.sender_name,
            player_id: m.sender_id,
            message: m.message,
          })),
        },
        timestamp: new Date(parsed.start_time_ms).toISOString(),
      };

      trySetDate(new Date(parsed.start_time_ms));

      internalReplayData = mapped;
      onSetReplayData?.(mapped);
    } catch (e) {
      lastParseError = String(e);
      console.error("Replay parse failed", e);
    } finally {
      loading = false;
    }
  };

  let gameDate = $state<Date | null>(null);

  const isValidDate = (d: Date) => {
    return !isNaN(d.getTime()) && d.getFullYear() > 2000;
  };

  const trySetDate = (val: Date | null | undefined) => {
    if (!val || (gameDate && isValidDate(gameDate))) return; // Already have a valid date

    if (isValidDate(val)) {
      gameDate = val;
    }
  };

  let relativeTime = $derived.by(() => {
    if (!gameDate || !timeAgo) return null;
    return timeAgo.format(gameDate);
  });

  onMount(async () => {
    trySetDate(match.timestamp);

    // Attempt to grab a replay timestamp immediately (non-blocking)
    try {
      const replays = await match.replays;
      const replay = replays.anyReplay;
      trySetDate(replay?.timestamp);
    } catch (e) {
      // Silent; date still may come from parse
    }

    // Kick off parse; if it later populates internalReplayData we derive date
    maybeParseReplay();

    try {
      const lang =
        typeof navigator !== "undefined" ? navigator.language : "en-US";
      timeAgo = new TimeAgo(lang);
    } catch {}
  });
</script>

<tr
  class="group hover:bg-muted/50 transition-all duration-300"
  class:blurred={isBlurred}
>
  <style>
    .blurred {
      filter: blur(2px) grayscale(0.7);
      opacity: 0.6;
    }
    .blurred:hover {
      filter: blur(0px) grayscale(0);
      opacity: 1;
    }
  </style>
  <td class="font-medium">
    {#if gameDate && relativeTime}
      <Tooltip.Root>
        <Tooltip.Trigger class="text-foreground">
          {relativeTime}
        </Tooltip.Trigger>
        <Tooltip.Content>
          <div class="text-sm">
            {gameDate.toLocaleString()}
          </div>
        </Tooltip.Content>
      </Tooltip.Root>
    {:else}
      <span class="text-muted-foreground text-sm">—</span>
    {/if}
  </td>
  <td><MapName name={match.map.displayName} /></td>
  <td>
    {#if match.thisPlayer?.race}
      <Race race={match.thisPlayer.race} />
    {:else}
      <span class="text-muted-foreground">?</span>
    {/if}
    <span class="text-muted-foreground">vs</span>
    {#if match.opponent?.race}
      <Race race={match.opponent.race} />
    {:else}
      <span class="text-muted-foreground">?</span>
    {/if}
  </td>
  <td>
    {#if match.opponent?.profileInfo?.gatewayId && match.opponent?.toon}
      <a
        href="/player/{match.opponent.profileInfo.gatewayId}/{match.opponent
          .toon}"
        class="text-primary hover:underline cursor-pointer"
        >{match.opponent.toon}</a
      >
    {:else if match.opponent?.toon}
      <span class="text-primary">{match.opponent.toon}</span>
    {:else}
      Unknown
    {/if}
  </td>
  <td>
    {#key match.id}
      {#if match.thisPlayer}
        {#if getMatchResult(match.thisPlayer) === "win"}
          <span
            class="px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
            >Win</span
          >
        {:else if getMatchResult(match.thisPlayer) === "loss"}
          <span
            class="px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
            >Loss</span
          >
        {:else}
          <span
            class="px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200"
            >Unknown</span
          >
        {/if}
      {:else}
        <span class="text-muted-foreground text-xs">—</span>
      {/if}
    {/key}
  </td>
  <td>
    {#if internalReplayData}
      <span class="text-sm"
        >{formatDuration(internalReplayData.parsed_data.game_duration_ms)}</span
      >
    {:else if loading}
      <Skeleton class="h-4 w-12 mx-auto" />
    {:else}
      <span class="text-muted-foreground text-sm">—</span>
    {/if}
  </td>
  <td>
    {#if match.thisPlayer?.profileInfo?.points?.delta !== undefined}
      <span
        class="text-sm font-medium {match.thisPlayer.profileInfo.points.delta >
        0
          ? 'text-green-600 dark:text-green-400'
          : match.thisPlayer.profileInfo.points.delta < 0
            ? 'text-red-600 dark:text-red-400'
            : 'text-muted-foreground'}"
        >{match.thisPlayer.profileInfo.points.delta > 0 ? "+" : ""}{match
          .thisPlayer.profileInfo.points.delta}</span
      >
    {:else}
      <span class="text-muted-foreground text-sm">—</span>
    {/if}
  </td>
  <td class="text-center">
    {#if internalReplayData && internalReplayData.parsed_data.chat_messages.length > 0}
      <Button
        onclick={() => {
          if (!internalReplayData) return;
          onOpenChat?.(internalReplayData.parsed_data.chat_messages);
        }}
        size="sm"
        variant="outline"
        class="h-7 my-1 text-xs cursor-pointer"
        >Chat ({internalReplayData.parsed_data.chat_messages.length})</Button
      >
    {:else if internalReplayData}
      <span class="text-muted-foreground text-xs">No chat</span>
    {:else if loading}
      <Skeleton class="h-7 w-12 mx-auto" />
    {:else}
      <span class="text-muted-foreground text-sm">—</span>
    {/if}
  </td>
  <td class="text-right">
    <Button
      onclick={downloadReplay}
      disabled={isDownloading}
      size="sm"
      variant="outline"
      class="h-7 my-1 text-xs cursor-pointer"
      >{isDownloading ? "Downloading..." : "Download"}</Button
    >
  </td>
</tr>
