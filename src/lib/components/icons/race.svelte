<script lang="ts">
  type Props = {
    race: string;
  };

  const { race }: Props = $props();

  const value = $state(race.toUpperCase().at(0) || "U");
</script>

{#if value === "T"}
  <span class="text-[#3091e7]">Terran</span>
{:else if value === "P"}
  <span class="text-[#f6aa33]">Protoss</span>
{:else if value === "Z"}
  <span class="text-[#925de8]">Zerg</span>
{:else if value === "R"}
  <span class="text-muted-foreground">Random</span>
{:else}
  <span class="text-muted-foreground">Unknown</span>
{/if}
