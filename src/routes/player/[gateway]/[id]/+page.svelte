<script lang="ts">
  import { onMount } from "svelte";

  import { afterNavigate } from "$app/navigation";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import BookmarkCheck from "@lucide/svelte/icons/bookmark-check";
  import Pencil from "@lucide/svelte/icons/pencil";

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
  import { getSettingsStore } from "@/lib/settingsStore.svelte";
  import { avatarOrDefault, debounce } from "@/lib/utils";

  import type { PageProps } from "./$types";

  const { data }: PageProps = $props();

  type Details = Awaited<typeof data.details>;

  // `data.details` is a streamed promise: SvelteKit completes the navigation
  // immediately (header + skeletons render right away) and we fill `details`
  // in one atomic write when it resolves. The single-bundle assignment is
  // what keeps us out of the original split-state bug class — every read of
  // `details` either sees null or the full bundle, never half of one and
  // half of another.
  let id = $derived(data.id);
  let gateway = $derived(data.gateway);
  let savedPlayersStore = $derived(data.savedPlayersStore);

  let details = $state<Details | null>(null);

  $effect(() => {
    const promise = data.details;
    details = null;
    let cancelled = false;
    promise.then((d) => {
      if (!cancelled) details = d;
    });
    return () => {
      cancelled = true;
    };
  });

  let profile = $derived(details?.profile ?? null);
  let ranking = $derived(details?.ranking ?? null);
  let otherRankings = $derived(details?.otherRankings ?? []);

  const settingsStorePromise = getSettingsStore();

  let avatar = $derived(avatarOrDefault(ranking?.avatar));

  let hideShortMatches = $state(false);

  let isSaved = $derived(
    profile?.auroraId ? savedPlayersStore.isSaved(profile.auroraId) : false,
  );
  let savedDetails = $derived(
    profile?.auroraId ? savedPlayersStore.getPlayer(profile.auroraId) : null,
  );

  let showEditAliasOpen = $state(false);
  let editAliasValue = $state("");

  onMount(async () => {
    try {
      const store = await settingsStorePromise;
      hideShortMatches = store.settings.hideShortReplays;
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

  // Build the canonical profile list from a fully-resolved details bundle.
  // Taking the bundle as an argument (instead of reading reactive state)
  // means callers can't accidentally pass a half-loaded view of the data.
  const knownProfilesFrom = (
    d: Details,
    routeId: string,
    routeGateway: string,
  ) => [
    {
      toon: routeId,
      gateway: Number.parseInt(routeGateway),
      lastViewed: Date.now(),
      race: d.ranking?.featureRace,
      avatarUrl: avatarOrDefault(d.ranking?.avatar),
    },
    ...d.otherRankings.map((r) => ({
      toon: r.toon,
      gateway:
        typeof r.gatewayId === "string"
          ? Number.parseInt(r.gatewayId)
          : Number(r.gatewayId),
      lastViewed: Date.now(),
      race: r.featureRace,
      avatarUrl: avatarOrDefault(r.avatar),
    })),
  ];

  const toggleSave = () => {
    if (!details || !profile?.auroraId) return;
    if (isSaved) {
      savedPlayersStore.removePlayer(profile.auroraId);
    } else {
      const profiles = knownProfilesFrom(details, id, gateway);
      savedPlayersStore.savePlayer(profile.auroraId, id, profiles[0]);
      savedPlayersStore.setProfiles(profile.auroraId, profiles);
    }
  };

  const openEditAlias = () => {
    if (savedDetails) {
      editAliasValue = savedDetails.alias;
      showEditAliasOpen = true;
    }
  };

  const saveAlias = () => {
    if (profile?.auroraId) {
      savedPlayersStore.renamePlayer(profile.auroraId, editAliasValue);
      showEditAliasOpen = false;
    }
  };

  // Per-navigation post-load action: sync the saved record once the streamed
  // bundle resolves. Route id/gateway and the savedPlayersStore are captured
  // up front so that even if the user has already navigated away by the time
  // the promise resolves, we sync the *right* player's record (or do
  // nothing, if `details` belongs to an aborted navigation).
  afterNavigate(() => {
    const promise = data.details;
    const routeId = data.id;
    const routeGateway = data.gateway;
    const store = data.savedPlayersStore;
    promise.then((d) => {
      if (d.profile.auroraId && store.isSaved(d.profile.auroraId)) {
        store.setProfiles(
          d.profile.auroraId,
          knownProfilesFrom(d, routeId, routeGateway),
        );
      }
    });
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

<div class="w-full h-[100vh] overflow-y-scroll scroll-smooth pb-8">
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
      {#if details === null}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          {#each Array(3) as _}
            <div class="flex items-center gap-3 p-3 rounded-md bg-background">
              <Skeleton class="w-8 h-8 rounded-md" />
              <div class="min-w-0 flex-1 space-y-1">
                <Skeleton class="h-4 w-24" />
                <Skeleton class="h-3 w-16" />
              </div>
            </div>
          {/each}
        </div>
      {:else if otherRankings.length > 0}
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

    <MatchesTable {hideShortMatches} {profile} loading={!profile || !ranking} />
  </div>
</div>
