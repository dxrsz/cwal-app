<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { listen } from '@tauri-apps/api/event';
	import { onDestroy } from 'svelte';

	let unlisten: () => void;

	(async () => {
		unlisten = await listen('scr-event', (event: any) => {
			console.log('click event', event.payload);

			if (event.payload.ProfileSelect) {
				ourAlias = event.payload.ProfileSelect.alias;
				ourGateway = event.payload.ProfileSelect.gateway;
			} else if (event.payload.MatchFound) {
				player1Alias = event.payload.MatchFound.player1.alias;
				player1Gateway = event.payload.MatchFound.player1.gateway;
				player2Alias = event.payload.MatchFound.player2.alias;
				player2Gateway = event.payload.MatchFound.player2.gateway;
				map = event.payload.MatchFound.map
			}
		});
	})();

	onDestroy(() => {
		if (unlisten) {
			unlisten();
		}
	});

	$: ourAlias = null;
	$: ourGateway = null;

	$: player1Alias = null;
	$: player1Gateway = null;

	$: player2Alias = null;
	$: player2Gateway = null;

	$: map = null;

	invoke('init_process');
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	<table border=1>
		<tbody>
		<tr>
			<td>Our Alias</td>
			<td>{ourAlias}</td>
		</tr>
		<tr>
			<td>Our Gateway</td>
			<td>{ourGateway}</td>
		</tr>
		</tbody>
	</table>
</section>
<section>
	<table border=1>
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
