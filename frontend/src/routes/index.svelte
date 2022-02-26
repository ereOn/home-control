<script context="module" lang="ts">
	export const prerender = true;
</script>

<script lang="ts">
	import AlarmStatus from '$lib/AlarmStatus.svelte';
</script>

<svelte:head>
	<title>Home</title>
</svelte:head>

<section>
	<AlarmStatus />
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 1;
	}
</style>
