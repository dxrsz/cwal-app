<script lang="ts">
  import { onMount } from "svelte";

  import { afterNavigate, goto } from "$app/navigation";
  import { page } from "$app/state";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import BookmarkCheck from "@lucide/svelte/icons/bookmark-check";
  import Pencil from "@lucide/svelte/icons/pencil";
  import type { GravaticBooster, Ranking } from "gravatic-booster";

  import CountryFlag from "@/lib/components/CountryFlag.svelte";
  import MatchesTable from "@/lib/components/MatchesTable.svelte";
  import Race from "@/lib/components/icons/race.svelte";
  import Rank from "@/lib/components/icons/rank.svelte";
  import * as Avatar from "@/lib/components/ui/avatar";
  import Button from "@/lib/components/ui/button/button.svelte";
  import * as Dialog from "@/lib/components/ui/dialog";
  import Input from "@/lib/components/ui/input/input.svelte";
  import Label from "@/lib/components/ui/label/label.svelte";
  import { Skeleton } from "@/lib/components/ui/skeleton";
  import { Switch } from "@/lib/components/ui/switch";
  import {
    type SavedPlayersStore,
    getSavedPlayersStore,
  } from "@/lib/savedPlayersStore.svelte";
  import { getGb, sleep } from "@/lib/scApi.svelte";
  import { getSettingsStore } from "@/lib/settingsStore.svelte";
  import { avatarOrDefault, debounce } from "@/lib/utils";

  import type { PageProps } from "./$types";

  const { data }: PageProps = $props();

  let id: string = $derived(data.id);
  let gateway: string = $derived(data.gateway);

  const gb = getGb();
  const settingsStorePromise = getSettingsStore();
  const savedPlayersStorePromise = getSavedPlayersStore();

  let savedPlayersStore: SavedPlayersStore | null = $state(null);
  let profile: Awaited<
    ReturnType<
      typeof GravaticBooster.prototype.minimalAccountWithGamesPlayedLastWeek
    >
  > | null = $state(null);
  let ranking: Ranking | null = $state(null);
  let otherRankings: Ranking[] = $state([]);
  let avatar = $derived.by(() => avatarOrDefault(ranking?.avatar));

  // Matches now owned/fetched within MatchesTable; we keep a callback reference to trigger loading
  let matchesTableFetchMore: (() => Promise<boolean>) | null = null;

  let scrollableDiv: HTMLDivElement | null = $state(null);
  let scrollTimeout: number | null = null;
  let hideShortMatches = $state(false);

  // Saved player state
  let isSaved = $derived.by(() => {
    if (!savedPlayersStore || !profile?.auroraId) return false;
    return savedPlayersStore.isSaved(profile.auroraId);
  });

  let savedDetails = $derived.by(() => {
    if (!savedPlayersStore || !profile?.auroraId) return null;
    return savedPlayersStore.getPlayer(profile.auroraId);
  });

  let showEditAliasOpen = $state(false);
  let editAliasValue = $state("");

  onMount(async () => {
    try {
      const store = await settingsStorePromise;
      hideShortMatches = store.settings.hideShortReplays;
      savedPlayersStore = await savedPlayersStorePromise;
    } catch (e) {
      console.error("Failed to load settings for player page", e);
    }
  });

  const updateHideShortMatches = debounce(async (hide: boolean) => {
    const settingsStore = await settingsStorePromise;
    if (hide !== settingsStore.settings.hideShortReplays) {
      settingsStore.updateHideShortReplays(hide);
    }
  }, 500);

  $effect(() => {
    updateHideShortMatches(hideShortMatches);
  });

  // Helper to extract known profiles from current state
  const getAllKnownProfiles = () => {
    const currentProps = {
      toon: id,
      gateway: Number.parseInt(gateway),
      lastViewed: Date.now(),
      race: ranking?.featureRace,
      avatarUrl: avatarOrDefault(ranking?.avatar),
    };
    const others = otherRankings.map((r) => ({
      toon: r.toon,
      gateway:
        typeof r.gatewayId === "string"
          ? Number.parseInt(r.gatewayId)
          : Number(r.gatewayId),
      lastViewed: Date.now(),
      race: r.featureRace,
      avatarUrl: avatarOrDefault(r.avatar),
    }));
    return [currentProps, ...others];
  };

  // Auto-add profiles if player is saved
  $effect(() => {
    if (
      savedPlayersStore &&
      profile?.auroraId &&
      savedPlayersStore.isSaved(profile.auroraId)
    ) {
      savedPlayersStore.setProfiles(profile.auroraId, getAllKnownProfiles());
    }
  });

  const toggleSave = () => {
    if (!savedPlayersStore || !profile?.auroraId) return;
    if (isSaved) {
      savedPlayersStore.removePlayer(profile.auroraId);
    } else {
      const initialProfiles = getAllKnownProfiles();
      // savePlayer creates the player record if needed
      savedPlayersStore.savePlayer(profile.auroraId, id, initialProfiles[0]);
      // setProfiles ensures the full list is synced (removing any old ones if re-saving, and adding all new ones)
      savedPlayersStore.setProfiles(profile.auroraId, initialProfiles);
    }
  };

  const openEditAlias = () => {
    if (savedDetails) {
      editAliasValue = savedDetails.alias;
      showEditAliasOpen = true;
    }
  };

  const saveAlias = () => {
    if (savedPlayersStore && profile?.auroraId) {
      savedPlayersStore.renamePlayer(profile.auroraId, editAliasValue);
      showEditAliasOpen = false;
    }
  };

  const fetchMoreMatches = async () => {
    if (!matchesTableFetchMore) return false;
    try {
      return await matchesTableFetchMore();
    } catch (e) {
      console.error("Error invoking MatchesTable fetchMore:", e);
      return false;
    }
  };

  const hasScrollbar = () => {
    if (!scrollableDiv) return false;
    return scrollableDiv.scrollHeight > scrollableDiv.clientHeight;
  };

  const fetchUntilScrollbarOrEnd = async () => {
    const maxIterations = 50;
    const maxTimeMs = 30000;
    const startTime = Date.now();
    let iterations = 0;
    while (iterations < maxIterations) {
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
        if (!fetchedMatches) break;
        await sleep(100); // allow DOM update
        if (hasScrollbar()) break;
      } catch (error) {
        console.error("Error in fetchUntilScrollbarOrEnd:", error);
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
    if (!scrollableDiv) return;
    if (scrollTimeout) clearTimeout(scrollTimeout);
    scrollTimeout = setTimeout(() => {
      if (!scrollableDiv) return;
      const { scrollHeight, scrollTop, clientHeight } = scrollableDiv;
      const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
      if (distanceFromBottom <= 200) {
        fetchMoreMatches();
      }
    }, 100);
  };

  afterNavigate(async () => {
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
      try {
        const acct = await profile.requestedProfile?.accountRankings(
          leaderboard.id,
        );
        if (acct && acct.rankings) {
          const reqGw = Number.parseInt(gateway);
          otherRankings = acct.rankings.filter(
            (r) => !(r.toon === id && Number(r.gatewayId) === reqGw),
          );
        }
      } catch (e) {
        console.warn("Failed to load other rankings", e);
      }
      // Wait for MatchesTable to register its fetcher then load initial entries
      const start = Date.now();
      while (!matchesTableFetchMore && Date.now() - start < 5000) {
        await sleep(50);
      }
      if (matchesTableFetchMore) {
        try {
          await fetchUntilScrollbarOrEnd();
        } catch (matchError) {
          console.error("Failed to load initial matches:", matchError);
        }
      } else {
        console.warn("MatchesTable fetcher not ready; skipping initial fetch");
      }
    } catch (error) {
      console.error("Failed to load player data:", error);
      goto(`/error?from=${encodeURIComponent(page.url.pathname)}`);
    }
  });

  const winPercentage = $derived.by(() => {
    if (ranking?.wins !== undefined && ranking?.losses !== undefined) {
      if (ranking.losses === 0) return "100%";
      return `${Math.round((ranking.wins / (ranking.wins + ranking.losses)) * 100)}%`;
    }
    return "N/A";
  });
</script>

<svelte:head>
  <title>Player - {id} @ {gateway}</title>
  <meta name="description" content="Player details page" />
</svelte:head>

<Dialog.Root bind:open={showEditAliasOpen}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <Dialog.Title>Edit Alias</Dialog.Title>
      <Dialog.Description>
        Make changes to the player alias here. Click save when you're done.
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid grid-cols-4 items-center gap-4">
        <Label for="name" class="text-right">Alias</Label>
        <Input id="name" bind:value={editAliasValue} class="col-span-3" />
      </div>
    </div>
    <Dialog.Footer>
      <Button type="submit" onclick={saveAlias}>Save changes</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>

{#key id + gateway}
  <div
    class="w-full h-[100vh] overflow-y-scroll scroll-smooth pb-8"
    onscroll={onScroll}
    bind:this={scrollableDiv}
  >
    <div class="p-6 space-y-6">
      <div class="bg-muted/20 rounded-lg p-6">
        <div class="flex items-start justify-between gap-6">
          <div class="flex items-start gap-6">
            <Avatar.Root class="w-20 h-20 flex-shrink-0 rounded-md">
              <Avatar.Image
                src={avatar}
                alt="Player Avatar"
                class="rounded-md"
              />
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
                {#if savedPlayersStore}
                  <Button
                    variant="outline"
                    size="icon"
                    class="h-9 w-9 cursor-pointer"
                    onclick={toggleSave}
                  >
                    {#if isSaved}
                      <BookmarkCheck class="h-4 w-4 text-green-500" />
                    {:else}
                      <Bookmark class="h-4 w-4" />
                    {/if}
                  </Button>
                  {#if isSaved}
                    <div class="flex items-center gap-2">
                      <span class="text-sm text-muted-foreground"
                        >Saved as: {savedDetails?.alias}</span
                      >
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-6 w-6"
                        onclick={openEditAlias}
                      >
                        <Pencil class="h-3 w-3" />
                      </Button>
                    </div>
                  {/if}
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
                  {ranking.rating} MMR
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
              <div class="text-lg font-bold">{winPercentage}</div>
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

      <div class="bg-muted/20 rounded-lg p-4">
        <h2 class="text-sm font-medium mb-1">Other profiles</h2>
        {#if otherRankings.length > 0}
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {#each otherRankings as r}
              <a
                href="/player/{r.gatewayId}/{r.toon}"
                class="flex items-center gap-3 p-3 rounded-md bg-background hover:bg-muted transition-colors"
                onclick={(e) => {
                  /* handled by default nav */
                }}
              >
                <Avatar.Root class="w-8 h-8 rounded-md">
                  <Avatar.Image
                    src={avatarOrDefault(r.avatar)}
                    alt={r.toon}
                    class="rounded-md"
                  />
                </Avatar.Root>
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-medium truncate max-w-[12rem]"
                      >{r.toon}</span
                    >
                    {#if r.featureRace}
                      <Race race={r.featureRace} />
                    {/if}
                  </div>
                  <div
                    class="text-xs text-muted-foreground flex items-center gap-2 mt-0.5"
                  >
                    {#if r.tier}
                      <Rank rank={r.tier} />
                    {/if}
                    {#if r.rating}
                      <span>{r.rating} MMR</span>
                    {/if}
                    <span>{r.gateway.name}</span>
                  </div>
                </div>
              </a>
            {/each}
          </div>
        {:else}
          <p class="text-xs text-muted-foreground">
            No other profiles on this account.
          </p>
        {/if}
      </div>

      <div class="flex items-center justify-between bg-muted/20 rounded-lg p-4">
        <div class="space-y-1">
          <label class="text-sm font-medium" for="hide-short-matches"
            >Blur short matches</label
          >
          <p class="text-xs text-muted-foreground">
            Blur matches shorter than 1 minute from the list
          </p>
        </div>
        <Switch
          id="hide-short-matches"
          bind:checked={hideShortMatches}
          class="cursor-pointer"
        />
      </div>

      <MatchesTable
        {hideShortMatches}
        {profile}
        loading={!profile || !ranking}
        onFetcherReady={(fn) => (matchesTableFetchMore = fn)}
      />
    </div>
  </div>
{/key}
