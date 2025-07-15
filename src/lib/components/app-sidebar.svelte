<script lang="ts">
  import MapIcon from "@lucide/svelte/icons/map";
  import LadderIcon from "@lucide/svelte/icons/waves-ladder";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
 
  const items = [
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
  ];
</script>

<Sidebar.Root>
  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each items as item (item.title)}
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
</Sidebar.Root>
