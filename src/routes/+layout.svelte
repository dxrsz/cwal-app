<script lang="ts">
  import "@/app.css";

  import TimeAgo from "javascript-time-ago";
  import en from "javascript-time-ago/locale/en.json";
  import { ModeWatcher } from "mode-watcher";
  import { Toaster } from "svelte-sonner";

  import AppSidebar from "@/lib/components/AppSidebar.svelte";
  import TitleBar from "@/lib/components/TitleBar.svelte";
  import * as Sidebar from "@/lib/components/ui/sidebar";
  import { configureReceiveBackendEvents } from "@/lib/scrState.svelte";

  try {
    TimeAgo.addDefaultLocale(en as any);
  } catch {}

  let { children } = $props();

  $effect.pre(() => {
    const unlisten = configureReceiveBackendEvents();
    return async () => {
      (await unlisten)();
    };
  });
</script>

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
