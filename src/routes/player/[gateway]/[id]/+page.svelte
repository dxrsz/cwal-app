<script lang="ts">
  import { getGb } from "@/lib/scApi.svelte";
  import type { PageProps } from "./$types";
  import { onMount } from "svelte";
  import type { GravaticBooster, Match, Ranking } from "gravatic-booster";
  import * as Card from "@/lib/components/ui/card";
  import { avatarOrDefault } from "@/lib/utils";

  const { data }: PageProps = $props();
  const { id, gateway } = data;

  const gb = getGb();

  let profile: Awaited<
    ReturnType<
      typeof GravaticBooster.prototype.minimalAccountWithGamesPlayedLastWeek
    >
  > | null = $state(null);
  let ranking: Ranking | null = $state(null);
  let avatar = $derived(avatarOrDefault(ranking?.avatar));
  let matchesGenerator: AsyncGenerator<Match, void, void> | null = null;
  let matches: Match[] = $state([]);

  const loadMoreMatches = async () => {
    if (!matchesGenerator) {
      return;
    }
    const next = await matchesGenerator.next();
    if (next.value) {
      matches.push(next.value);
    }
  };

  onMount(async () => {
    const _gb = await gb;
    profile = await _gb.minimalAccountWithGamesPlayedLastWeek(id, {
      gateway: Number.parseInt(gateway),
    });
    const leaderboard = await _gb.leaderboard({
      seasonId: profile.currentSeason,
    });
    ranking = (await profile.requestedProfile?.ranking(leaderboard.id)) ?? null;
    matchesGenerator = (await profile.requestedProfile?.ladderGames()) ?? null;
    await loadMoreMatches();
  });
</script>

<svelte:head>
  <title>Player - {id} @ {gateway}</title>
  <meta name="description" content="Player details page" />
</svelte:head>

<div class="container p-2">
  <div class="flex">
    <img src={avatar} />
    {id}
  </div>
  <div>
    {#each matches as match}
      {match.timestamp}
      {match.closedSlots}
      {match.flags}
      {match.gameSpeed}
      {match.hostName}
      {match.netTurnRate}
      {match.map}
      {match.id}
      {match.name}
      {match.players}
      {match.gameId}
    {/each}
  </div>
</div>

<style>
</style>
