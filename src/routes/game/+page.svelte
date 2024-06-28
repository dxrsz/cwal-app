<script lang="ts">
	import { scrState } from '$lib/scrState.svelte';
	import { getGb } from '$lib/scApi.svelte';
	import { jsonParseMap } from '$lib/util';

	scrState.opponent = {
		alias: 'dontseriousgame',
		gateway: 11
	};

	scrState.user = {
		alias: 'dontseriousgame',
		gateway: 11
	};

	let opponentStats = $derived.by(async () => {
		if (scrState.opponent?.alias && scrState.opponent?.gateway) {
			const gb = await getGb();
			return await gb?.mapStatsByToon(scrState.opponent.alias, {
				gateway: scrState.opponent.gateway
			});
		}
	});

	$inspect(jsonParseMap(opponentStats));
</script>

<div class="w-full h-full flex items-center justify-center">
	{#if !scrState.gameRunning}
		<div class="flex flex-col items-center justify-center w-full">
			<span class="loading loading-infinity loading-lg" />
			<h2>StarCraft does not appear to be running.</h2>
		</div>
	{:else if scrState.opponent && scrState.user}
		<div class="flex flex-col items-center justify-center w-full">
			<h2>Game Found!</h2>
			<h3>Playing against {scrState.opponent.alias} on {scrState.opponent.gateway}</h3>
			<div class="flex flex-col items-center justify-center w-full">
				<h4>Opponent Stats</h4>
				<div>
					{#await opponentStats}
						<span class="loading loading-infinity loading-lg" />
					{:then opponentStats}
						{#each opponentStats.get('1v1')?.entries() as [season, seasonStats]}
							{season}
							<table>
									<thead>
										<tr>
											<th>Map</th>
											<th>Games</th>
											<th>Wins</th>
										</tr>
									</thead>
								{#each seasonStats.entries() as [mapId, byRace]}
									<tbody>
										<tr>
											<td>{mapId}</td>
											<td>{byRace.get('zerg').games}</td>
											<td>{byRace.get('zerg').wins}</td>
										</tr>
									</tbody>
								{/each}
							</table>
						{/each}
					{:catch error}
						<p>{error.message}</p>
					{/await}
				</div>
			</div>
		</div>
	{:else}
		<div class="flex flex-col items-center justify-center w-full">
			<span class="loading loading-infinity loading-lg" />
			<h2>Waiting for a game to be found.</h2>
		</div>
	{/if}
</div>
