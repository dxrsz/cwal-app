<script>
  import { MinusIcon, SquareIcon, XIcon } from "@lucide/svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import { Button } from "@/lib/components/ui/button";
  import * as Menubar from "@/lib/components/ui/menubar";

  const window = getCurrentWindow();

  let version = $state(/** @type {string | null} */ (null));
  getVersion().then((v) => (version = v));
</script>

<div class="z-10">
  <Menubar.Root
    class="flex justify-between items-center bg-clip-border rounded-b-none"
    data-tauri-drag-region
  >
    <div data-tauri-drag-region class="flex-1"></div>
    <div data-tauri-drag-region class="flex-1 flex justify-center">
      <span
        data-tauri-drag-region
        class="text-sm font-medium text-muted-foreground"
      >
        cwal.gg{#if version}<span class="ml-1.5 text-xs opacity-70"
            >v{version}</span
          >{/if}
      </span>
    </div>
    <div data-tauri-drag-region class="flex-1 flex justify-end">
      <Menubar.Menu>
        <Button
          onclick={() => window.minimize()}
          variant="ghost"
          size="icon"
          class="size-8"
        >
          <MinusIcon />
        </Button>
        <Button
          onclick={() => window.toggleMaximize()}
          variant="ghost"
          size="icon"
          class="size-8"
        >
          <SquareIcon />
        </Button>
        <Button
          onclick={() => window.close()}
          variant="ghost"
          size="icon"
          class="size-8"
        >
          <XIcon />
        </Button>
      </Menubar.Menu>
    </div>
  </Menubar.Root>
</div>
