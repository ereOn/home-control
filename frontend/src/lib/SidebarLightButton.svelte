<script lang="ts">
	export let icon: string;
	export let name: string;

	let status = getStatus();

	async function getStatus() {
		return await (await fetch('/api/v1/light/' + name)).json();
	}

	async function setStatus(s) {
		try {
			const res = await fetch('/api/v1/light/' + name, {
				method: 'POST',
				headers: { Accept: 'Application/json', 'Content-Type': 'application/json' },
				body: JSON.stringify(s)
			});

			if (res.headers.get('content-type') == 'application/json') {
				status = Promise.resolve(await res.json());
			} else {
				console.error(await res.text());
			}
		} catch (err) {
			console.error(err);
		}
	}

	import { onMount } from 'svelte';

	let poller;

	function setupPoller() {
		if (poller) {
			clearInterval(poller);
		}

		poller = setInterval(doPoll, 1000);
	}

	async function doPoll() {
		// Let's not assign the refresh promise to status or the UI will flicker.
		let newStatus = await getStatus();
		status = Promise.resolve(newStatus);
	}

	onMount(() => {
		setupPoller();
	});

	import Icon from '@iconify/svelte';
</script>

{#await status}
	<button class="loading"><Icon {icon} style="font-size: 48px" /></button>
{:then status}
	<button class={status ? 'on' : ''} on:click={() => setStatus(!status)}
		><Icon {icon} style="font-size: 48px" /></button
	>
{:catch error}
	<span class="error">{error}</span>
{/await}

<style lang="scss">
	button {
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 8px;
		background-color: var(--button-background-color-off);
		background: radial-gradient(
			ellipse at center,
			white 20%,
			var(--button-background-center-color-off) 100%
		);
		color: var(--button-text-color);
		height: 80px;
		width: 80px;
		font-weight: 800;
		font-size: 120%;
		user-select: none;

		&.on {
			background: radial-gradient(
				ellipse at center,
				white 20%,
				var(--button-background-center-color-on) 100%
			);
		}

		&:active {
			filter: brightness(0.8);
		}
	}
</style>
