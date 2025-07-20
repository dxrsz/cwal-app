<script lang="ts">
  import { goto } from "$app/navigation";
  import PlayerSearch from "@/lib/components/PlayerSearch.svelte";

  const handlePlayerSelect = (name: string, gateway: string) => {
    goto(`/player/${gateway}/${encodeURIComponent(name)}`);
  };
</script>

<svelte:head>
  <title>Player Search - CWAL</title>
  <meta name="description" content="Search for StarCraft Remastered players" />
</svelte:head>

<div class="flex flex-col items-center justify-center min-h-[60vh] px-4">
  <div class="w-full max-w-2xl">
    <div class="text-center mb-8">
      <h1 class="text-4xl font-bold text-foreground mb-2">Player Search</h1>
    </div>

    <div class="w-full">
      <PlayerSearch onPlayerSelect={handlePlayerSelect} />
    </div>
  </div>
</div>
