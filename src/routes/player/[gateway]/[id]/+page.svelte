<script lang="ts">
  import { afterNavigate } from "$app/navigation";
  import type { GravaticBooster, Match, Ranking } from "gravatic-booster";

  import MapName from "@/lib/components/MapName.svelte";
  import Race from "@/lib/components/icons/race.svelte";
  import Rank from "@/lib/components/icons/rank.svelte";
  import * as Avatar from "@/lib/components/ui/avatar";
  import * as Card from "@/lib/components/ui/card";
  import { Skeleton } from "@/lib/components/ui/skeleton";
  import * as Table from "@/lib/components/ui/table";
  import * as Tooltip from "@/lib/components/ui/tooltip";
  import { getGb, sleep } from "@/lib/scApi.svelte";
  import { avatarOrDefault } from "@/lib/utils";

  import type { PageProps } from "./$types";

  const { data }: PageProps = $props();

  let id: string = $derived(data.id);
  let gateway: string = $derived(data.gateway);

  const MATCH_FETCH_NUM = 15;

  const gb = getGb();

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
    switch (result) {
      case "win":
        return "Win";
      case "loss":
        return "Loss";
      case "draw":
        return "Draw";
      case "undecided":
        return "Undecided";
      default:
        return "Unknown";
    }
  };

  const fetchMoreMatches = async () => {
    if (!matchesGenerator) {
      return false;
    }
    let fetchedAny = false;
    for (let i = 0; i < MATCH_FETCH_NUM; ++i) {
      const next = await matchesGenerator.next();
      if (next.done) {
        break;
      }

      matches.push(next.value!);
      fetchedAny = true;
    }
    return fetchedAny;
  };

  const hasScrollbar = () => {
    if (!scrollableDiv) return false;
    return scrollableDiv.scrollHeight > scrollableDiv.clientHeight;
  };

  const fetchUntilScrollbarOrEnd = async () => {
    let shouldContinue = true;
    while (shouldContinue) {
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

  afterNavigate(async () => {
    matches = [];
    const _gb = await gb;
    profile = await _gb.minimalAccountWithGamesPlayedLastWeek(id, {
      gateway: Number.parseInt(gateway),
    });
    const leaderboard = await _gb.leaderboard({
      seasonId: profile.currentSeason,
    });
    ranking = (await profile.requestedProfile?.ranking(leaderboard.id)) ?? null;
    matchesGenerator = (await profile.requestedProfile?.ladderGames()) ?? null;
    await fetchUntilScrollbarOrEnd();
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
      <!-- Player Header -->
      <div class="bg-muted/20 rounded-lg p-4">
        <div class="flex items-center gap-4">
          <Avatar.Root class="w-14 h-14">
            <Avatar.Image src={avatar} alt="Player Avatar" />
            <Avatar.Fallback class="text-base font-bold"
              >{id.slice(0, 2).toUpperCase()}</Avatar.Fallback
            >
          </Avatar.Root>
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <h1 class="text-2xl font-bold">{id}</h1>
              {#if ranking?.featureRace}
                <div class="px-2 py-1 bg-background rounded text-sm">
                  <Race race={ranking.featureRace} />
                </div>
              {/if}
            </div>
            <p class="text-sm text-muted-foreground">
              {ranking?.gateway?.name || `Gateway ${gateway}`}
            </p>
          </div>
        </div>
      </div>

      <!-- Player Stats - Compact -->
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
              <p class="text-xs text-muted-foreground">
                {ranking.rating
                  ? `${ranking.rating} Rating`
                  : "Current Ranking"}
              </p>
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
          <!-- Compact Skeleton Loading -->
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

      <!-- Recent Matches -->
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
                  <Table.Head class="text-center">Result</Table.Head>
                  <Table.Head class="text-right">MMR</Table.Head>
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
                        {:else}
                          <div class="flex items-center gap-2">
                            <Tooltip.Root>
                              <Tooltip.Trigger
                                class="w-4 h-4 rounded-full border border-muted-foreground/50 text-muted-foreground/70 hover:text-muted-foreground hover:border-muted-foreground text-xs flex items-center justify-center cursor-help"
                              >
                                ?
                              </Tooltip.Trigger>
                              <Tooltip.Content>
                                <p class="text-sm">
                                  API returned corrupt timestamp for this match
                                </p>
                              </Tooltip.Content>
                            </Tooltip.Root>
                          </div>
                        {/if}
                      </Table.Cell>
                      <Table.Cell>
                        <MapName name={match.map.displayName} />
                      </Table.Cell>
                      <Table.Cell>
                        <div class="flex items-center gap-1 text-sm">
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
                        </div>
                      </Table.Cell>
                      <Table.Cell>
                        {#if match.opponent?.toon}
                          <a
                            href="/player/{match.opponent?.profileInfo
                              ?.gatewayId}/{match.opponent?.toon}"
                            class="text-primary hover:underline"
                          >
                            {match.opponent.toon}
                          </a>
                        {:else}
                          Unknown
                        {/if}
                      </Table.Cell>
                      <Table.Cell class="text-center">
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
                      <Table.Cell class="text-right">
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
                    </Table.Row>
                  {/each}
                {:else if profile && ranking}
                  <Table.Row>
                    <Table.Cell
                      colspan={6}
                      class="text-center py-8 text-muted-foreground"
                    >
                      No matches found for this player
                    </Table.Cell>
                  </Table.Row>
                {:else}
                  <!-- Skeleton Loading Rows -->
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
                      <Table.Cell class="text-right">
                        <Skeleton class="h-4 w-8 ml-auto" />
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
