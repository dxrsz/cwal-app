<script lang="ts">
  import PlayerSearch from "@/lib/components/PlayerSearch.svelte";
  import * as Table from "@/lib/components/ui/table";
  import { getGb } from "@/lib/scApi.svelte";
  import { debounce } from "@/lib/utils";
  import { Effect } from "@tauri-apps/api/window";
  import type { Ranking } from "gravatic-booster";
  import { onMount, type SvelteComponent } from "svelte";

  const gb = getGb();

  const avatarOrDefault = (avatar?: string) => {
    return (
      avatar ||
      "https://scrassets.classic.blizzard.com/avatar-icons/S1/d2edd0800865e3a5e0c0c86898010f90.png"
    );
  };

  const LOAD_MORE_DELTA = 100;

  let rankings: Array<Ranking> = $state([]);
  let fetching = $state(false);
  let rankingsGenerator: AsyncGenerator<Ranking, void, unknown> | null = null;
  let scrollableDiv: HTMLDivElement;

  onMount(() => {
    // todo; a better method would be to detect if we don't have a scrollbar yet and keep fetching until that happens
    fetchMoreRankings();
  });

  const onScroll = () => {
    const { scrollHeight, scrollTop, clientHeight } = scrollableDiv;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
    if (distanceFromBottom <= 200) {
      fetchMoreRankings();
    }
  };

  const fetchMoreRankings = debounce(async () => {
    const _gb = await gb;
    fetching = true;
    try {
      if (!rankingsGenerator) {
        rankingsGenerator = _gb.rankings({});
      }

      for (let i = 0; i < LOAD_MORE_DELTA; ++i) {
        const next = await rankingsGenerator.next();
        if (next.done) {
          break;
        }

        rankings.push(next.value!);
      }
    } finally {
      fetching = false;
    }
  }, 250);
</script>

<svelte:head>
  <title>Home</title>
  <meta name="description" content="Svelte demo app" />
</svelte:head>

<div
  class="block p-2 w-full h-[100vh] overflow-y-scroll scroll-smooth unanchored"
  onscroll={onScroll}
  bind:this={scrollableDiv}
>
  <PlayerSearch />
  <Table.Root>
    <Table.Caption>A list of your recent invoices.</Table.Caption>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-[100px]">Rank</Table.Head>
        <Table.Head>Player</Table.Head>
        <Table.Head>MMR</Table.Head>
        <Table.Head>Wins</Table.Head>
        <Table.Head>Losses</Table.Head>
        <Table.Head>Disconnects</Table.Head>
        <Table.Head>Race</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each rankings as ranking}
        <Table.Row>
          <Table.Cell>{ranking.rank}</Table.Cell>
          <Table.Cell class="min-w-max">
            <a
              class="min-w-max"
              href="/player/{ranking.gatewayId}/{ranking.toon}"
            >
              <div class="flex items-center space-x-3 min-w-max">
                <img
                  src={avatarOrDefault(ranking.avatar)}
                  alt="avatar"
                  class="w-12 h-12"
                />
                <div>
                  <div class="font-bold">{ranking.toon}</div>
                  <div class="text-sm opacity-50">{ranking.gateway.name}</div>
                </div>
              </div>
            </a>
          </Table.Cell>
          <Table.Cell>{ranking.rating} {ranking.tier}</Table.Cell>
          <Table.Cell>{ranking.wins}</Table.Cell>
          <Table.Cell>{ranking.losses}</Table.Cell>
          <Table.Cell>{ranking.disconnects}</Table.Cell>
          <Table.Cell>{ranking.featureRace}</Table.Cell>
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>

<style>
  .unanchored {
    overflow-anchor: none;
  }
</style>
