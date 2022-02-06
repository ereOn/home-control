<script lang="ts">
	export let ledPath: string;
	export let ledColor: string;

	let ledStatus = getLedStatus();

	async function getLedStatus() {
		return await (await fetch(ledPath)).json();
	}

	async function setLedStatus(status) {
		try {
			const res = await fetch(ledPath, {
				method: 'POST',
				headers: { Accept: 'Application/json', 'Content-Type': 'application/json' },
				body: JSON.stringify(status)
			});

			if (res.headers.get('content-type') == 'application/json') {
				const status = await res.json();

				ledStatus = Promise.resolve(status);
			} else {
				console.error(await res.text());
			}
		} catch (err) {
			console.error(err);
		}
	}
</script>

{#await ledStatus}
	<span>Waiting.About..</span>
{:then ledStatus}
	<button style="--color: {ledColor}" on:click={() => setLedStatus(!ledStatus)}
		>{ledStatus ? 'ON' : 'OFF'}</button
	>
{:catch error}
	<span class="error">{error}</span>
{/await}

<style>
	button {
		width: 100px;
		height: 100px;
		background-color: var(--color);
	}

	.error {
		color: red;
		font: bold;
	}
</style>
