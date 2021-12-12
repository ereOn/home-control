<script lang="ts">
	export let ledPath: string;
	export let ledColor: string;

	let ledStatus = getLedStatus();

	async function getLedStatus() {
		return await (await fetch(ledPath)).json();
	}

	function setLedStatus(status) {
		fetch(ledPath, {
			method: 'POST',
			headers: { Accept: 'Application/json', 'Content-Type': 'application/json' },
			body: JSON.stringify(status)
		})
			.then((res) => res.json())
			.then((status: boolean) => {
				ledStatus = Promise.resolve(status);
			})
			.catch((error) => {
				console.error(error);
			});
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
