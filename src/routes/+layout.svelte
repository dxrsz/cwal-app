<script lang="ts">
  import { configureReceiveTauriEvents } from "@/lib/scrState.svelte";
  import { ModeWatcher } from "mode-watcher";
  import * as Sidebar from "@/lib/components/ui/sidebar";
  import AppSidebar from "@/lib/components/app-sidebar.svelte";
  import PlayerSearch from "@/lib/components/PlayerSearch.svelte";

  let { children } = $props();

  $effect.pre(() => {
    const unlisten = configureReceiveTauriEvents();
    return async () => {
      (await unlisten)();
    };
  });
</script>

<ModeWatcher />

<Sidebar.Provider>
  <AppSidebar />
  <main class="w-full p-2">
    <PlayerSearch />
    {@render children?.()}
  </main>
</Sidebar.Provider>
