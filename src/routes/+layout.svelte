<script lang="ts">
  import { ModeWatcher } from "mode-watcher";
  import { Toaster } from "svelte-sonner";

  import TitleBar from "@/lib/components/TitleBar.svelte";
  import AppSidebar from "@/lib/components/app-sidebar.svelte";
  import * as Sidebar from "@/lib/components/ui/sidebar";
  import { configureReceiveTauriEvents } from "@/lib/scrState.svelte";

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

<div class="flex flex-col w-full h-[100vh] overflow-hidden">
  <TitleBar />
  <div class="flex flex-1 min-h-0">
    <Sidebar.Provider class="!min-h-0 h-full">
      <AppSidebar />
      <main class="w-full">
        {@render children?.()}
      </main>
    </Sidebar.Provider>
  </div>
</div>

<Toaster />
