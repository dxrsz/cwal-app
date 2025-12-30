<script lang="ts">
  import { onMount, untrack } from "svelte";

  import type { GravaticBooster, Match } from "gravatic-booster";

  import MatchRow from "@/lib/components/Match.svelte";
  import * as Card from "@/lib/components/ui/card";
  import * as Dialog from "@/lib/components/ui/dialog";
  import { Skeleton } from "@/lib/components/ui/skeleton";
  import * as Table from "@/lib/components/ui/table";
  import * as Tooltip from "@/lib/components/ui/tooltip";

  interface ChatMessage {
    timestamp: number;
    player: string;
    player_id: number;
    message: string;
  }

  interface ReplayDataMinimal {
    parsed_data: {
      game_duration_ms: number;
      chat_messages: ChatMessage[];
    };
    timestamp: string; // ISO string
  }

  type MinimalAccount = Awaited<
    ReturnType<
      typeof GravaticBooster.prototype.minimalAccountWithGamesPlayedLastWeek
    >
  >;

  interface Props {
    matches?: Match[];
    profile?: MinimalAccount | null;
    loading?: boolean;
    hideShortMatches?: boolean;
    onFetchReplayData?: (match: Match) => void;
    onFetcherReady?: (fetcher: () => Promise<boolean>) => void;
  }

  let {
    matches = [],
    profile = null,
    loading = false,
    hideShortMatches = false,
    onFetcherReady,
  }: Props = $props();

  // Internal reactive matches state
  let internalMatches: Match[] = $state(untrack(() => [...matches]));

  const MATCH_FETCH_NUM = 15;
  let matchesGenerator: AsyncGenerator<Match, void, void> | null = null;
  let internalLoading = $state(false);
  let lastProfileRef: MinimalAccount | null = null;

  const ensureGenerator = async () => {
    if (matchesGenerator || !profile?.requestedProfile) return;
    try {
      matchesGenerator =
        (await profile.requestedProfile?.ladderGames()) ?? null;
    } catch (e) {
      console.error("Failed to create matches generator", e);
    }
  };

  $effect(() => {
    if (profile !== lastProfileRef) {
      internalMatches = [];
      matchesGenerator = null;
      lastProfileRef = profile;
      void ensureGenerator();
    }
  });

  const fetchMore = async (): Promise<boolean> => {
    if (internalLoading) return false;
    await ensureGenerator();
    if (!matchesGenerator) return false;
    internalLoading = true;
    let fetchedAny = false;
    try {
      for (let i = 0; i < MATCH_FETCH_NUM; i++) {
        const next = await matchesGenerator.next();
        if (next.done) break;
        internalMatches.push(next.value);
        // force reactivity in case push doesn't trigger
        internalMatches = internalMatches;
        fetchedAny = true;
      }
    } catch (e) {
      console.error("Match fetch failed (continuing):", e);
    } finally {
      internalLoading = false;
    }
    return fetchedAny;
  };

  onMount(() => {
    onFetcherReady?.(fetchMore);
  });

  let replayDataCache = $state(new Map<string, ReplayDataMinimal>());
  let selectedChatMessages: ChatMessage[] = $state([]);
  let showChatDialog = $state(false);

  // Determine if a match should be blurred
  const isMatchBlurred = (match: Match) => {
    if (!hideShortMatches) return false;
    const key = match.name || match.id;
    const replayData = replayDataCache.get(key);
    if (!replayData?.parsed_data?.game_duration_ms) return false;
    return replayData.parsed_data.game_duration_ms < 60000;
  };
  const setReplayData = (key: string, data: ReplayDataMinimal) => {
    if (!key) return;
    const newCache = new Map(replayDataCache);
    newCache.set(key, data);
    // trigger reactivity
    replayDataCache = newCache;
  };

  const showChatMessages = (chatMessages: ChatMessage[]) => {
    selectedChatMessages = chatMessages;
    showChatDialog = true;
  };
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>Recent Matches</Card.Title>
    <Card.Description>Latest ladder games for this player</Card.Description>
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
          {#if internalMatches.length > 0}
            {#each internalMatches as match}
              {#key match.id}
                <MatchRow
                  {match}
                  replayData={replayDataCache.get(match.name || match.id) ||
                    undefined}
                  isBlurred={isMatchBlurred(match)}
                  onOpenChat={(msgs) => showChatMessages(msgs)}
                  onSetReplayData={(data) =>
                    setReplayData(match.name || match.id, data)}
                />
              {/key}
            {/each}
          {:else if !loading && !internalLoading}
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
