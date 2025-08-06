<script lang="ts">
  type Props = {
    race: string;
  };

  const { race }: Props = $props();

  const value = $derived(race[0].toUpperCase());
</script>

{#if value === "T"}
  <span class="text-[#3091e7]">Terran</span>
{:else if value === "P"}
  <span class="text-[#f6aa33]">Protoss</span>
{:else}
  <span class="text-[#925de8]">Zerg</span>
{/if}
