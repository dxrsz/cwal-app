<script lang="ts">
  import * as Tooltip from "@/lib/components/ui/tooltip";
  import { getCountryFlag, getCountryName } from "@/lib/countries";

  interface Props {
    countryCode: string;
    class?: string;
  }

  const { countryCode, class: className = "" }: Props = $props();
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    <div
      class="flex items-center justify-end px-3 py-1.5 bg-background rounded-md text-sm gap-2 cursor-help shadow-sm hover:bg-muted/50 transition-colors {className}"
    >
      <span class="font-medium">{countryCode.toUpperCase()}</span>
      <span class="text-xl flag-emoji">{getCountryFlag(countryCode)}</span>
    </div>
  </Tooltip.Trigger>
  <Tooltip.Content>
    <p>{getCountryName(countryCode)}</p>
  </Tooltip.Content>
</Tooltip.Root>
