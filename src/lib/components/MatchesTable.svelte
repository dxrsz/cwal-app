<script lang="ts">
  import type { Match } from "gravatic-booster";

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

  interface Props {
    matches: Match[];
    loading?: boolean;
    hideShortMatches?: boolean;
    onFetchReplayData?: (match: Match) => void; // retained for external override if needed
  }

  let {
    matches = [],
    loading = false,
    hideShortMatches = false,
  }: Props = $props();

  let replayDataCache = $state(new Map<string, ReplayDataMinimal>());
  let selectedChatMessages: ChatMessage[] = $state([]);
  let showChatDialog = $state(false);

  // Filter matches based on hideShortMatches setting (reacts when cache updates)
  let filteredMatches = $derived.by(() => {
    if (!hideShortMatches) return matches;

    return matches.filter((match) => {
      // If we don't have replay data yet, include the match (it will be filtered later when data loads)
      if (!match.name || !replayDataCache.has(match.name)) {
        return true;
      }

      const replayData = replayDataCache.get(match.name);
      if (!replayData?.parsed_data?.game_duration_ms) {
        return true; // Include if we can't determine duration
      }

      // Hide matches shorter than 1 minute (60000ms)
      return replayData.parsed_data.game_duration_ms >= 60000;
    });
  });
  const setReplayData = (matchName: string, data: ReplayDataMinimal) => {
    if (!matchName) return;
    replayDataCache.set(matchName, data);
    replayDataCache = replayDataCache; // trigger reactivity
  };

  const showChatMessages = (chatMessages: ChatMessage[]) => {
    selectedChatMessages = chatMessages;
    showChatDialog = true;
  };

  // Downloading logic moved into MatchRow for decoupling

  // No effect needed; MatchRow handles fetching on mount
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
          {#if filteredMatches.length > 0}
            {#each filteredMatches as match}
              <MatchRow
                {match}
                replayData={match.name
                  ? replayDataCache.get(match.name)
                  : undefined}
                onOpenChat={(msgs) => showChatMessages(msgs)}
                onSetReplayData={(data) =>
                  setReplayData(match.name || "", data)}
              />
            {/each}
          {:else if !loading}
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
