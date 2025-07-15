<script lang="ts">
  import { configureReceiveTauriEvents } from "@/lib/scrState.svelte";
  import { ModeWatcher } from "mode-watcher";
  import * as Sidebar from "@/lib/components/ui/sidebar";
  import AppSidebar from "@/lib/components/app-sidebar.svelte";

  let { children } = $props();

  $effect.pre(() => {
    const unlisten = configureReceiveTauriEvents();
    return async () => {
      (await unlisten)();
    };
  });
</script>

<ModeWatcher />

<svelte:head>
  <title>CWAL Desktop App</title>
  <meta name="description" content="CWAL Desktop App" />
</svelte:head>

<Sidebar.Provider>
  <AppSidebar />
  <main class="w-full">
    {@render children?.()}
  </main>
</Sidebar.Provider>
