<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import { Button } from "@/lib/components/ui/button";

  const goHome = () => {
    goto("/search");
  };

  const goBack = () => {
    if (window.history.length > 1) {
      window.history.back();
    } else {
      goto("/search");
    }
  };
</script>

<svelte:head>
  <title>Error - CWAL</title>
  <meta name="description" content="Something went wrong" />
</svelte:head>

<div class="flex flex-col items-center justify-center min-h-[60vh] px-4">
  <div class="w-full max-w-2xl text-center">
    <div class="mb-6">
      <h1 class="text-9xl font-bold text-muted-foreground/30 leading-none">
        Oops
      </h1>
    </div>

    <div class="mb-4">
      <h2 class="text-4xl font-bold text-foreground mb-2">
        Something went wrong
      </h2>
      <p class="text-lg text-muted-foreground">
        We encountered an unexpected error. Please try again or return to the
        main page.
      </p>
    </div>

    <div class="mb-8">
      <p
        class="text-sm text-muted-foreground/80 font-mono bg-muted px-3 py-2 rounded-md inline-block"
      >
        {page.url.pathname}
      </p>
    </div>

    <div class="flex flex-col sm:flex-row gap-3 justify-center">
      <Button
        onclick={goBack}
        variant="outline"
        class="min-w-[120px] cursor-pointer"
      >
        Go Back
      </Button>
      <Button onclick={goHome} class="min-w-[120px] cursor-pointer">
        Search Players
      </Button>
    </div>

    <div class="mt-12 text-sm text-muted-foreground">
      <p>
        Looking for player stats? Try the
        <button
          onclick={goHome}
          class="text-primary hover:underline font-medium cursor-pointer"
        >
          player search
        </button>
        to get started.
      </p>
    </div>
  </div>
</div>
