<script lang="ts">
  const props = $props<{
    apiRequests: number;
    replaysDownloaded: number;
    label?: string;
  }>();

  let apiRequests = $derived(props.apiRequests);
  let replaysDownloaded = $derived(props.replaysDownloaded);
  let label = $derived(props.label ?? "Requests & Replays");

  type Level = "low" | "med" | "high";

  // Derive a severity / color style tier by simple thresholds
  const classify = (n: number): Level => {
    if (n >= 1000) return "high";
    if (n >= 250) return "med";
    return "low";
  };

  let reqLevel = $derived(classify(apiRequests));
  let replayLevel = $derived(classify(replaysDownloaded));

  const colorMap: Record<Level, { bg: string; text: string; accent: string }> =
    {
      low: { bg: "bg-slate-600", text: "text-white", accent: "bg-slate-500" },
      med: { bg: "bg-blue-600", text: "text-white", accent: "bg-blue-500" },
      high: {
        bg: "bg-purple-700",
        text: "text-white",
        accent: "bg-purple-600",
      },
    };

  let containerColor = $derived.by(() => {
    if (reqLevel === "high" || replayLevel === "high") return colorMap.high;
    if (reqLevel === "med" || replayLevel === "med") return colorMap.med;
    return colorMap.low;
  });
</script>

<div
  class="relative overflow-hidden rounded-lg px-4 py-3 {containerColor.bg}"
  aria-label={label}
>
  <div class="absolute inset-0 opacity-15">
    <div class="h-full w-full {containerColor.accent} animate-pulse"></div>
  </div>
  <div class="relative flex items-center gap-6">
    <p
      class="text-sm font-medium {containerColor.text} truncate flex-1 min-w-0"
    >
      {label}
    </p>
    <div class="flex items-center gap-4 {containerColor.text}">
      <div class="flex items-baseline gap-1">
        <span class="text-[10px] uppercase tracking-wide opacity-70">API</span>
        <span class="text-sm font-semibold leading-none tabular-nums"
          >{apiRequests}</span
        >
      </div>
      <div class="flex items-baseline gap-1">
        <span class="text-[10px] uppercase tracking-wide opacity-70"
          >Replays</span
        >
        <span class="text-sm font-semibold leading-none tabular-nums"
          >{replaysDownloaded}</span
        >
      </div>
    </div>
  </div>
</div>
