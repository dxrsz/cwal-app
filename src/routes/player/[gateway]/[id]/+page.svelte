<script lang="ts">
  import { afterNavigate, goto } from "$app/navigation";
  import { page } from "$app/stores";
  import type { GravaticBooster, Match, Ranking } from "gravatic-booster";

  import CountryFlag from "@/lib/components/CountryFlag.svelte";
  import MatchesTable from "@/lib/components/MatchesTable.svelte";
  import Race from "@/lib/components/icons/race.svelte";
  import Rank from "@/lib/components/icons/rank.svelte";
  import * as Avatar from "@/lib/components/ui/avatar";
  import { Skeleton } from "@/lib/components/ui/skeleton";
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
  let loadingMatches = $state(false);
  let scrollTimeout: number | null = null;

  const fetchMoreMatches = async () => {
    if (!matchesGenerator) {
      return false;
    }

    if (loadingMatches) {
      return false;
    }

    loadingMatches = true;
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
        // Log error for background loading when we already have some matches
        console.error("Failed to load more matches:", error);
      }
    } finally {
      loadingMatches = false;
    }
    return fetchedAny;
  };

  const hasScrollbar = () => {
    if (!scrollableDiv) return false;
    return scrollableDiv.scrollHeight > scrollableDiv.clientHeight;
  };

  const fetchUntilScrollbarOrEnd = async () => {
    const maxIterations = 50; // Prevent infinite loops
    const maxTimeMs = 30000; // 30 second timeout
    const startTime = Date.now();

    let iterations = 0;

    while (iterations < maxIterations) {
      // Check timeout
      if (Date.now() - startTime > maxTimeMs) {
        console.warn(
          "fetchUntilScrollbarOrEnd timed out after",
          maxTimeMs,
          "ms",
        );
        break;
      }

      try {
        const fetchedMatches = await fetchMoreMatches();
        if (!fetchedMatches) {
          break;
        }

        // Give DOM time to update
        await sleep(100);

        if (hasScrollbar()) {
          break;
        }
      } catch (error) {
        console.error("Error in fetchUntilScrollbarOrEnd:", error);
        // Break the loop on error to prevent infinite retry
        break;
      }

      iterations++;
    }

    if (iterations >= maxIterations) {
      console.warn(
        "fetchUntilScrollbarOrEnd reached maximum iterations:",
        maxIterations,
      );
    }
  };

  const onScroll = () => {
    if (!scrollableDiv) {
      return;
    }

    // Debounce scroll events to prevent excessive calls
    if (scrollTimeout) {
      clearTimeout(scrollTimeout);
    }

    scrollTimeout = setTimeout(() => {
      if (!scrollableDiv) return;

      const { scrollHeight, scrollTop, clientHeight } = scrollableDiv;
      const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

      if (distanceFromBottom <= 200 && !loadingMatches) {
        fetchMoreMatches();
      }
    }, 100); // 100ms debounce
  };

  afterNavigate(async () => {
    matches = [];

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
            <Avatar.Root class="w-20 h-20 flex-shrink-0 rounded-md">
              <Avatar.Image src={avatar} alt="Player Avatar" class="rounded-md" />
              <Avatar.Fallback class="text-xl font-bold rounded-md"
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
              {#if profile?.battleTag}
                <div class="text-sm text-muted-foreground mb-2">
                  <span class="font-mono">{profile.battleTag}</span>
                </div>
              {/if}
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

      <MatchesTable {matches} loading={!profile || !ranking} />
    </div>
  </div>
{/key}
