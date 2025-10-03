<script lang="ts">
  import MapIcon from "@lucide/svelte/icons/map";
  import SearchIcon from "@lucide/svelte/icons/search";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import LadderIcon from "@lucide/svelte/icons/waves-ladder";

  import RequestsStatus from "@/lib/components/RequestsStatus.svelte";
  import StarCraftStatus from "@/lib/components/StarCraftStatus.svelte";
  import * as Sidebar from "@/lib/components/ui/sidebar";
  import { getLimitsStore } from "@/lib/limits.svelte";

  const items = [
    {
      title: "Search",
      url: "/search",
      icon: SearchIcon,
    },
    {
      title: "Ladder",
      url: "/ladder",
      icon: LadderIcon,
    },
    {
      title: "Maps",
      url: "/maps",
      icon: MapIcon,
    },
    {
      title: "Settings",
      url: "/settings",
      icon: SettingsIcon,
    },
  ];

  let apiRequestsData = getLimitsStore();

  let numApiRequests = $derived(apiRequestsData.numApiRequests);
  let numReplayDownloads = $derived(apiRequestsData.numReplayDownloads);
</script>

<Sidebar.Root>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each items as item}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton>
                {#snippet child({ props })}
                  <a href={item.url} {...props}>
                    <item.icon />
                    <span>{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <RequestsStatus
      apiRequests={numApiRequests}
      replaysDownloaded={numReplayDownloads}
      label="Usage"
    />
    <StarCraftStatus />
  </Sidebar.Footer>
</Sidebar.Root>
