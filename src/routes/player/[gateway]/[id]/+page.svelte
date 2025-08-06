<script lang="ts">
  import { afterNavigate, goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
  import type {
    GravaticBooster,
    Match,
    Ranking,
    Replay,
  } from "gravatic-booster";
  import { toast } from "svelte-sonner";

  import CountryFlag from "@/lib/components/CountryFlag.svelte";
  import MapName from "@/lib/components/MapName.svelte";
  import Race from "@/lib/components/icons/race.svelte";
  import Rank from "@/lib/components/icons/rank.svelte";
  import * as Avatar from "@/lib/components/ui/avatar";
  import { Button } from "@/lib/components/ui/button";
  import * as Card from "@/lib/components/ui/card";
  import * as Dialog from "@/lib/components/ui/dialog";
  import { Skeleton } from "@/lib/components/ui/skeleton";
  import * as Table from "@/lib/components/ui/table";
  import * as Tooltip from "@/lib/components/ui/tooltip";
  import { getGb, sleep } from "@/lib/scApi.svelte";
  import { getSettingsStore } from "@/lib/settingsStore.svelte";
  import { avatarOrDefault } from "@/lib/utils";

  import type { PageProps } from "./$types";

  interface ChatMessage {
    timestamp: number;
    player: string;
    player_id: number;
    message: string;
  }

  interface ReplayData {
    id: string;
    parsed_data: {
      id: string;
      map_name: string;
      game_duration_ms: number;
      players: Array<{
        id: number;
        name: string;
        race: string;
        color: {
          name: string;
          rgb: number;
        };
        apm: number;
      }>;
      chat_messages: ChatMessage[];
    };
    cached: boolean;
    timestamp: string;
  }

  const { data }: PageProps = $props();

  let id: string = $derived(data.id);
  let gateway: string = $derived(data.gateway);

  const MATCH_FETCH_NUM = 15;

  const gb = getGb();
  const settingsStore = getSettingsStore();

  let profile: Awaited<
    ReturnType<
      typeof GravaticBooster.prototype.minimalAccountWithGamesPlayedLastWeek
    >
  > | null = $state(null);
  let ranking: Ranking | null = $state(null);
  let avatar = $derived.by(() => avatarOrDefault(ranking?.avatar));
  let matchesGenerator: AsyncGenerator<Match, void, void> | null = null;
  let matches: Match[] = $state([]);
  let scrollableDiv: HTMLDivElement | null = $state(null);
  let downloadingReplays = $state(new Set<string>());
  let replayDataCache = $state(new Map<string, ReplayData>());
  let loadingReplayData = $state(new Set<string>());
  let selectedChatMessages: ChatMessage[] = $state([]);
  let showChatDialog = $state(false);

  const getMatchResult = (player: any) => {
    if (
      player?.profileInfo?.points?.delta !== undefined &&
      player.profileInfo.points.delta !== 0
    ) {
      return player.profileInfo.points.delta > 0 ? "win" : "loss";
    }
    return player?.result || "unknown";
  };

  const getResultDisplay = (result: string) => {
    return result.charAt(0).toUpperCase() + result.slice(1);
  };

  const fetchMoreMatches = async () => {
    if (!matchesGenerator) {
      return false;
    }
    let fetchedAny = false;
    try {
      for (let i = 0; i < MATCH_FETCH_NUM; ++i) {
        const next = await matchesGenerator.next();
        if (next.done) {
          break;
        }

        matches.push(next.value!);
        fetchedAny = true;
      }
    } catch (error) {
      console.error("Failed to fetch more matches:", error);

      // If we have no matches yet, this is likely a fundamental error (like EntityNotFoundError)
      // so redirect to error page. If we already have some matches, just show a toast.
      if (matches.length === 0) {
        console.error(
          "Initial match loading failed, redirecting to error page",
        );
        goto(`/error?from=${encodeURIComponent($page.url.pathname)}`);
        return false;
      } else {
        // Show toast for background loading errors when we already have some matches
        toast.error("Failed to load more matches", {
          description: "There was an error loading additional match data.",
        });
      }
    }
    return fetchedAny;
  };

  const hasScrollbar = () => {
    if (!scrollableDiv) return false;
    return scrollableDiv.scrollHeight > scrollableDiv.clientHeight;
  };

  const sanitizeFilename = (filename: string): string => {
    return filename.replace(/[<>:"/\\|?*]/g, "_");
  };

  const getRaceInitial = (race: string | undefined): string => {
    if (!race || race.length === 0) return "U";
    return race[0].toUpperCase();
  };

  const generateReplayFilename = (match: Match, replay: Replay): string => {
    const timestamp = match.timestamp ?? replay.timestamp;
    const formattedDate = timestamp?.toISOString() || "unknown";
    const p1Alias = match.thisPlayer?.toon || "Unknown";
    const p1Race = getRaceInitial(match.thisPlayer?.race);
    const p2Alias = match.opponent?.toon || "Unknown";
    const p2Race = getRaceInitial(match.opponent?.race);

    return sanitizeFilename(
      `${formattedDate}_${p1Alias}(${p1Race})_vs_${p2Alias}(${p2Race}).rep`,
    );
  };

  const formatDuration = (durationMs: number): string => {
    const totalSeconds = Math.floor(durationMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}m${seconds.toString().padStart(2, "0")}s`;
  };

  const fetchReplayData = async (match: Match): Promise<ReplayData | null> => {
    if (!match.name) return null;

    const matchKey = match.name;

    // Check if already cached
    if (replayDataCache.has(matchKey)) {
      return replayDataCache.get(matchKey)!;
    }

    // Check if already loading
    if (loadingReplayData.has(matchKey)) {
      return null;
    }

    loadingReplayData.add(matchKey);
    // Force reactivity update for loading state
    loadingReplayData = new Set(loadingReplayData);

    try {
      const replays = await match.replays;
      const replay = replays.anyReplay;

      if (!replay) {
        return null;
      }

      const gameId = encodeURIComponent(match.name);
      const replayUrl = encodeURIComponent(replay.url);
      const url = `https://repser.cwal.gg/replay/${gameId}?url=${replayUrl}`;

      const response = await tauriFetch(url);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const replayData: ReplayData = await response.json();
      replayDataCache.set(matchKey, replayData);
      // Force reactivity update for cache
      replayDataCache = new Map(replayDataCache);
      return replayData;
    } catch (error) {
      console.error("Failed to fetch replay data:", error);
      return null;
    } finally {
      loadingReplayData.delete(matchKey);
      // Force reactivity update for loading state
      loadingReplayData = new Set(loadingReplayData);
    }
  };

  const showChatMessages = (chatMessages: ChatMessage[]) => {
    selectedChatMessages = chatMessages;
    showChatDialog = true;
  };

  const fetchUntilScrollbarOrEnd = async () => {
    let shouldContinue = true;
    while (shouldContinue) {
      try {
        const fetchedMatches = await fetchMoreMatches();
        if (!fetchedMatches) {
          // No more matches available
          break;
        }

        // Give DOM time to update
        await sleep(100);

        if (hasScrollbar()) {
          // We now have enough content to scroll
          break;
        }
      } catch (error) {
        console.error("Error in fetchUntilScrollbarOrEnd:", error);
        // Break the loop on error to prevent infinite retry
        break;
      }
    }
  };

  const onScroll = () => {
    if (!scrollableDiv) {
      return;
    }

    const { scrollHeight, scrollTop, clientHeight } = scrollableDiv;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

    if (distanceFromBottom <= 200) {
      fetchMoreMatches();
    }
  };

  const downloadReplay = async (match: Match) => {
    if (!settingsStore.initialized) {
      toast.error("Settings not loaded yet. Please try again.");
      return;
    }

    const replayKey = `${match.id || match.timestamp?.getTime() || Math.random()}`;
    if (downloadingReplays.has(replayKey)) {
      return;
    }

    downloadingReplays.add(replayKey);
    downloadingReplays = new Set(downloadingReplays);

    try {
      const replays = await match.replays;
      const replay = replays.anyReplay;

      if (!replay) {
        toast.error("No replay available for this match");
        return;
      }

      const replayDownloadName = generateReplayFilename(match, replay);
      const result = await invoke<string>("download_file", {
        url: replay.url,
        destinationPath: settingsStore.settings.replayDownloadPath,
        filename: replayDownloadName,
      });

      toast.success(`Replay downloaded successfully`, {
        description: `Saved to: ${result}`,
      });
    } catch (error) {
      toast.error("Download failed", {
        description: String(error),
      });
    } finally {
      downloadingReplays.delete(replayKey);
      downloadingReplays = new Set(downloadingReplays);
    }
  };

  afterNavigate(async () => {
    matches = [];
    replayDataCache.clear();
    loadingReplayData.clear();

    try {
      const _gb = await gb;
      profile = await _gb.minimalAccountWithGamesPlayedLastWeek(id, {
        gateway: Number.parseInt(gateway),
      });
      const leaderboard = await _gb.leaderboard({
        seasonId: profile.currentSeason,
      });
      ranking =
        (await profile.requestedProfile?.ranking(leaderboard.id)) ?? null;
      matchesGenerator =
        (await profile.requestedProfile?.ladderGames()) ?? null;

      // Try to fetch initial matches
      try {
        await fetchUntilScrollbarOrEnd();
      } catch (matchError) {
        console.error("Failed to load initial matches:", matchError);
        // If we can't load any matches at all, redirect to error page
        goto(`/error?from=${encodeURIComponent($page.url.pathname)}`);
        return;
      }
    } catch (error) {
      console.error("Failed to load player data:", error);
      // Redirect to error page with current URL context
      goto(`/error?from=${encodeURIComponent($page.url.pathname)}`);
    }
  });

  // Fetch replay data for newly loaded matches
  $effect(() => {
    for (const match of matches) {
      if (
        match.name &&
        !replayDataCache.has(match.name) &&
        !loadingReplayData.has(match.name)
      ) {
        fetchReplayData(match);
      }
    }
  });
</script>

<svelte:head>
  <title>Player - {id} @ {gateway}</title>
  <meta name="description" content="Player details page" />
</svelte:head>

{#key id + gateway}
  <div
    class="w-full h-[100vh] overflow-y-scroll scroll-smooth"
    onscroll={onScroll}
    bind:this={scrollableDiv}
  >
    <div class="p-6 space-y-6">
      <div class="bg-muted/20 rounded-lg p-6">
        <div class="flex items-start justify-between gap-6">
          <div class="flex items-start gap-6">
            <Avatar.Root class="w-20 h-20 flex-shrink-0">
              <Avatar.Image src={avatar} alt="Player Avatar" />
              <Avatar.Fallback class="text-xl font-bold"
                >{id.slice(0, 2).toUpperCase()}</Avatar.Fallback
              >
            </Avatar.Root>

            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-4 mb-3">
                <h1 class="text-3xl font-bold text-foreground">{id}</h1>
                {#if ranking?.featureRace}
                  <div
                    class="px-3 py-1.5 bg-background rounded-md text-sm font-medium shadow-sm"
                  >
                    <Race race={ranking.featureRace} />
                  </div>
                {/if}
              </div>
            </div>
          </div>

          <div class="flex-shrink-0 space-y-3 text-right">
            {#if profile?.countryCode}
              <CountryFlag countryCode={profile.countryCode} />
            {/if}
          </div>
        </div>
      </div>

      <div class="bg-muted/20 rounded-lg p-4">
        {#if profile && ranking}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 text-center">
            <div class="space-y-1">
              <div class="flex items-center justify-center gap-2">
                <span class="text-lg font-bold"
                  >#{ranking.rank || "Unranked"}</span
                >
                {#if ranking.tier}
                  <Rank rank={ranking.tier} />
                {/if}
              </div>
              {#if ranking.rating}
                <span class="text-xs text-muted-foreground">
                  ${ranking.rating} MMR
                </span>
              {/if}
            </div>

            <div class="space-y-1">
              <div class="text-lg font-bold">
                {profile.requestedProfile?.numGamesLastWeek || 0}
              </div>
              <p class="text-xs text-muted-foreground">Games This Week</p>
            </div>

            <div class="space-y-1">
              <div class="text-lg font-bold">
                {ranking?.wins && ranking?.losses
                  ? `${Math.round((ranking.wins / (ranking.wins + ranking.losses)) * 100)}%`
                  : "N/A"}
              </div>
              <p class="text-xs text-muted-foreground">
                Win Rate ({ranking?.wins || 0}W/{ranking?.losses || 0}L)
              </p>
            </div>
          </div>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 text-center">
            <div class="space-y-1">
              <div class="flex items-center justify-center gap-2">
                <Skeleton class="h-6 w-16" />
                <Skeleton class="h-5 w-6" />
              </div>
              <Skeleton class="h-3 w-20 mx-auto" />
            </div>

            <div class="space-y-1">
              <Skeleton class="h-6 w-8 mx-auto" />
              <Skeleton class="h-3 w-20 mx-auto" />
            </div>

            <div class="space-y-1">
              <Skeleton class="h-6 w-12 mx-auto" />
              <Skeleton class="h-3 w-24 mx-auto" />
            </div>
          </div>
        {/if}
      </div>

      <Card.Root>
        <Card.Header>
          <Card.Title>Recent Matches</Card.Title>
          <Card.Description>
            Latest ladder games for this player
          </Card.Description>
        </Card.Header>
        <Card.Content>
          <Tooltip.Provider>
            <Table.Root>
              <Table.Header>
                <Table.Row>
                  <Table.Head>Date</Table.Head>
                  <Table.Head>Map</Table.Head>
                  <Table.Head>Matchup</Table.Head>
                  <Table.Head>Opponent</Table.Head>
                  <Table.Head>Result</Table.Head>
                  <Table.Head>Duration</Table.Head>
                  <Table.Head>MMR</Table.Head>
                  <Table.Head></Table.Head>
                  <Table.Head></Table.Head>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {#if matches.length > 0}
                  {#each matches as match}
                    <Table.Row class="hover:bg-muted/50">
                      <Table.Cell class="font-medium">
                        {#if match.timestamp}
                          <Tooltip.Root>
                            <Tooltip.Trigger
                              class="cursor-help underline decoration-dotted decoration-muted-foreground/40 hover:decoration-muted-foreground/60"
                            >
                              {match.timestamp.toLocaleDateString()}
                            </Tooltip.Trigger>
                            <Tooltip.Content>
                              <p class="text-sm">
                                {match.timestamp.toLocaleString()}
                              </p>
                            </Tooltip.Content>
                          </Tooltip.Root>
                        {:else if match.name && replayDataCache.has(match.name)}
                          {@const replayData = replayDataCache.get(match.name)}
                          {#if replayData?.timestamp}
                            {@const replayDate = new Date(replayData.timestamp)}
                            <Tooltip.Root>
                              <Tooltip.Trigger
                                class="cursor-help underline decoration-dotted decoration-muted-foreground/40 hover:decoration-muted-foreground/60"
                              >
                                {replayDate.toLocaleDateString()}
                              </Tooltip.Trigger>
                              <Tooltip.Content>
                                <p class="text-sm">
                                  {replayDate.toLocaleString()}
                                </p>
                              </Tooltip.Content>
                            </Tooltip.Root>
                          {:else}
                            <span class="text-muted-foreground text-sm">—</span>
                          {/if}
                        {:else if match.name && loadingReplayData.has(match.name)}
                          <Skeleton class="h-4 w-20" />
                        {:else}
                          <span class="text-muted-foreground text-sm">—</span>
                        {/if}
                      </Table.Cell>
                      <Table.Cell>
                        <MapName name={match.map.displayName} />
                      </Table.Cell>
                      <Table.Cell>
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
                      </Table.Cell>
                      <Table.Cell>
                        {#if match.opponent?.profileInfo?.gatewayId && match.opponent?.toon}
                          <a
                            href="/player/{match.opponent.profileInfo
                              .gatewayId}/{match.opponent.toon}"
                            class="text-primary hover:underline cursor-pointer"
                          >
                            {match.opponent.toon}
                          </a>
                        {:else if match.opponent?.toon}
                          <span class="text-primary">
                            <Tooltip.Root>
                              <Tooltip.Trigger
                                class="text-muted-foreground/70 hover:text-muted-foreground cursor-help"
                              >
                                {match.opponent.toon}
                              </Tooltip.Trigger>
                              <Tooltip.Content>
                                <p class="text-sm">
                                  This player's profile no longer exists
                                </p>
                              </Tooltip.Content>
                            </Tooltip.Root>
                          </span>
                        {:else}
                          Unknown
                        {/if}
                      </Table.Cell>
                      <Table.Cell>
                        {@const matchResult = getMatchResult(match.thisPlayer)}
                        <span
                          class="px-2 py-1 rounded-full text-xs font-medium {matchResult ===
                          'win'
                            ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                            : matchResult === 'loss'
                              ? 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                              : 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200'}"
                        >
                          {getResultDisplay(matchResult)}
                        </span>
                      </Table.Cell>
                      <Table.Cell>
                        {#if match.name && replayDataCache.has(match.name)}
                          {@const replayData = replayDataCache.get(match.name)}
                          {#if replayData}
                            <span class="text-sm">
                              {formatDuration(
                                replayData.parsed_data.game_duration_ms,
                              )}
                            </span>
                          {/if}
                        {:else if match.name && loadingReplayData.has(match.name)}
                          <Skeleton class="h-4 w-12 mx-auto" />
                        {:else}
                          <span class="text-muted-foreground text-sm">—</span>
                        {/if}
                      </Table.Cell>
                      <Table.Cell>
                        {#if match.thisPlayer?.profileInfo?.points?.delta !== undefined}
                          <span
                            class="text-sm font-medium {match.thisPlayer
                              .profileInfo.points.delta > 0
                              ? 'text-green-600 dark:text-green-400'
                              : match.thisPlayer.profileInfo.points.delta < 0
                                ? 'text-red-600 dark:text-red-400'
                                : 'text-muted-foreground'}"
                          >
                            {match.thisPlayer.profileInfo.points.delta > 0
                              ? "+"
                              : ""}{match.thisPlayer.profileInfo.points.delta}
                          </span>
                        {:else}
                          <span class="text-muted-foreground text-sm">—</span>
                        {/if}
                      </Table.Cell>
                      <Table.Cell class="text-center">
                        {#if match.name && replayDataCache.has(match.name)}
                          {@const replayData = replayDataCache.get(match.name)}
                          {#if replayData && replayData.parsed_data.chat_messages.length > 0}
                            <Button
                              onclick={() =>
                                showChatMessages(
                                  replayData.parsed_data.chat_messages,
                                )}
                              size="sm"
                              variant="outline"
                              class="h-7 px-2 text-xs"
                            >
                              Chat
                            </Button>
                          {:else if replayData}
                            <span class="text-muted-foreground text-xs"
                              >No chat</span
                            >
                          {/if}
                        {:else if match.name && loadingReplayData.has(match.name)}
                          <Skeleton class="h-7 w-12 mx-auto" />
                        {:else}
                          <span class="text-muted-foreground text-sm">—</span>
                        {/if}
                      </Table.Cell>
                      <Table.Cell class="text-right">
                        {@const replayKey = `${match.id || match.timestamp?.getTime() || Math.random()}`}
                        <Button
                          onclick={() => downloadReplay(match)}
                          disabled={downloadingReplays.has(replayKey)}
                          size="sm"
                          variant="outline"
                          class="cursor-pointer"
                        >
                          {downloadingReplays.has(replayKey)
                            ? "Downloading..."
                            : "Download"}
                        </Button>
                      </Table.Cell>
                    </Table.Row>
                  {/each}
                {:else if profile && ranking}
                  <Table.Row>
                    <Table.Cell
                      colspan={9}
                      class="text-center py-8 text-muted-foreground"
                    >
                      No matches found for this player
                    </Table.Cell>
                  </Table.Row>
                {:else}
                  {#each Array(5) as _}
                    <Table.Row>
                      <Table.Cell>
                        <Skeleton class="h-4 w-20" />
                      </Table.Cell>
                      <Table.Cell>
                        <Skeleton class="h-4 w-32" />
                      </Table.Cell>
                      <Table.Cell>
                        <div class="flex items-center gap-1">
                          <Skeleton class="h-4 w-12" />
                          <Skeleton class="h-4 w-6" />
                          <Skeleton class="h-4 w-12" />
                        </div>
                      </Table.Cell>
                      <Table.Cell>
                        <Skeleton class="h-4 w-24" />
                      </Table.Cell>
                      <Table.Cell class="text-center">
                        <Skeleton class="h-6 w-16 mx-auto" />
                      </Table.Cell>
                      <Table.Cell class="text-center">
                        <Skeleton class="h-4 w-12 mx-auto" />
                      </Table.Cell>
                      <Table.Cell class="text-right">
                        <Skeleton class="h-4 w-8 ml-auto" />
                      </Table.Cell>
                      <Table.Cell class="text-center">
                        <Skeleton class="h-7 w-12 mx-auto" />
                      </Table.Cell>
                      <Table.Cell class="text-right">
                        <Skeleton class="h-6 w-20 ml-auto" />
                      </Table.Cell>
                    </Table.Row>
                  {/each}
                {/if}
              </Table.Body>
            </Table.Root>
          </Tooltip.Provider>
        </Card.Content>
      </Card.Root>
    </div>
  </div>
{/key}

<!-- Chat Messages Dialog -->
<Dialog.Root bind:open={showChatDialog}>
  <Dialog.Content class="max-w-2xl max-h-[80vh] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Chat Messages</Dialog.Title>
      <Dialog.Description>
        In-game chat messages from this match
      </Dialog.Description>
    </Dialog.Header>
    <div class="space-y-3 mt-4">
      {#if selectedChatMessages.length > 0}
        {#each selectedChatMessages as message}
          <div class="border-l-2 border-primary/20 pl-3 py-2">
            <div
              class="flex items-center gap-2 text-sm text-muted-foreground mb-1"
            >
              <span class="font-medium text-foreground">{message.player}</span>
              <span>•</span>
              <span
                >{Math.floor(message.timestamp / 1000 / 60)}:{(
                  Math.floor(message.timestamp / 1000) % 60
                )
                  .toString()
                  .padStart(2, "0")}</span
              >
            </div>
            <p class="text-sm">{message.message}</p>
          </div>
        {/each}
      {:else}
        <p class="text-center text-muted-foreground py-8">
          No chat messages in this match
        </p>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
