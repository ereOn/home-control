<script lang="ts">
	const apiPath = '/api/v1/alarm';

	let status = getStatus();

	async function getStatus() {
		return await (await fetch(apiPath)).json();
	}
</script>

{#await status}
	<span>Waiting...</span>
{:then status}
	<h1 class={status ? 'disarmed' : 'armed'}>{status ? 'Disarmed' : 'Armed'}</h1>
{:catch error}
	<span class="error">{error}</span>
{/await}

<style>
	h1.disarmed {
		color: green;
	}

	h1.armed {
		color: red;
	}

	.error {
		color: red;
		font: bold;
	}
</style>
