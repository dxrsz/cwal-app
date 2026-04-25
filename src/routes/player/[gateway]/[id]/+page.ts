import { redirect } from "@sveltejs/kit";
import type { Ranking } from "gravatic-booster";

import { getSavedPlayersStore } from "@/lib/savedPlayersStore.svelte";
import { getGb } from "@/lib/scApi.svelte";

export const load = async ({ params, url }) => {
  const { gateway, id } = params;
  const gw = Number.parseInt(gateway);

  // Awaited: cheap singletons. The page can render the toon-name header
  // and the bookmark button immediately on navigation.
  const savedPlayersStore = await getSavedPlayersStore();

  // Streamed: every API hit lives inside one promise, so the page sees
  // either nothing yet or the full bundle — never partially-filled state.
  const details = (async () => {
    try {
      const gb = await getGb();
      const profile = await gb.minimalAccountWithGamesPlayedLastWeek(id, {
        gateway: gw,
      });
      const leaderboard = await gb.leaderboard({
        seasonId: profile.currentSeason,
      });
      const ranking =
        (await profile.requestedProfile?.ranking(leaderboard.id)) ?? null;

      let otherRankings: Ranking[] = [];
      try {
        const acct = await profile.requestedProfile?.accountRankings(
          leaderboard.id,
        );
        if (acct?.rankings) {
          otherRankings = acct.rankings.filter(
            (r) => !(r.toon === id && Number(r.gatewayId) === gw),
          );
        }
      } catch (e) {
        console.warn("Failed to load other rankings", e);
      }

      return { profile, ranking, otherRankings };
    } catch (e) {
      console.error("Failed to load player data:", e);
      throw redirect(302, `/error?from=${encodeURIComponent(url.pathname)}`);
    }
  })();

  return { id, gateway, savedPlayersStore, details };
};
