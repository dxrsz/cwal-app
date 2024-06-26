<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { listen } from '@tauri-apps/api/event';
	import { onDestroy } from 'svelte';
	import type { ScrEvent } from '$lib/tauri';
	import { SCApi, BroodWarConnection } from 'bw-web-api';

	let unlisten: () => void;
	let port: number | null = $state(null);

	type TauriEvent<T> = {
		payload: T;
	}

	(async () => {
		unlisten = await listen('scr-event', (event: TauriEvent<ScrEvent>) => {
			if ('ProfileSelect' in event.payload) {
				ourAlias = event.payload.ProfileSelect.alias;
				ourGateway = event.payload.ProfileSelect.gateway;
			} else if ('MatchFound' in event.payload) {
				player1Alias = event.payload.MatchFound.player1.alias;
				player1Gateway = event.payload.MatchFound.player1.gateway;
				player2Alias = event.payload.MatchFound.player2.alias;
				player2Gateway = event.payload.MatchFound.player2.gateway;
				map = event.payload.MatchFound.map;
			} else if ('WebServerRunning' in event.payload) {
				port = event.payload.WebServerRunning.port;
			}
		});
	})();

	onDestroy(() => {
		if (unlisten) {
			unlisten();
		}
	});

	const scapiFromPort: (port: number | null) => SCApi | null = (port: number | null) =>
		port != null ? new SCApi(new BroodWarConnection(`http://localhost:${port}`)) : null;

	const playerSearch = async (searchValue: string) => {
		if (!scapi || !searchValue || searchValue.length <= 3) {
			return;
		}

		try {
			searchResults = await scapi.leaderboardNameSearch(12960, searchValue);
		} catch (e) {
			searchResults = [];
			console.error(e);
		}
	};

	let ourAlias: string | null = $state(null);
	let ourGateway: number | null = $state(null);

	let player1Alias: string | null = $state(null);
	let player1Gateway: number | null = $state(null);

	let player2Alias: string | null = $state(null);
	let player2Gateway: number | null = $state(null);

	let map = $state(null);

	let scapi = $derived(scapiFromPort(port));

	let searchValue = $state('');
	let searchResults: Awaited<ReturnType<SCApi['leaderboardNameSearch']>> = $state([]);

	invoke('init_process');
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<div class="dropdown">
	<input type="text" class="grow" placeholder="Search" bind:value={searchValue} oninput={() => playerSearch(searchValue)} />
	<ul class="menu dropdown-content bg-base-100 rounded-box z-[1] w-52 p-2 shadow">
		{#each searchResults as searchResult}
			<li>{searchResult.name}</li>
		{/each}
	</ul>
</div>

<section>
	<table border="1">
		<tbody>
			<tr>
				<td>Our Alias</td>
				<td>{ourAlias}</td>
			</tr>
			<tr>
				<td>Our Gateway</td>
				<td>{ourGateway}</td>
			</tr>
			<tr>
				<td>Port</td>
				<td>{port}</td>
			</tr>
		</tbody>
	</table>
</section>
<section>
	<table border="1">
		<tbody>
			<tr>
				<td>Player 1</td>
				<td>{player1Alias}</td>
				<td>{player1Gateway}</td>
			</tr>
			<tr>
				<td>Player 2</td>
				<td>{player2Alias}</td>
				<td>{player2Gateway}</td>
			</tr>
			<tr>
				<td>Map</td>
				<td>{map}</td>
			</tr>
		</tbody>
	</table>
</section>

<style>
</style>
