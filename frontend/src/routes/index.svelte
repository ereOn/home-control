<script context="module" lang="ts">
	export const prerender = true;
</script>

<script lang="ts">
	import WeatherForecast from '$lib/WeatherForecast.svelte';
</script>

<svelte:head>
	<title>Home</title>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
	<link
		href="https://fonts.googleapis.com/css2?family=Lato:ital,wght@0,100;0,300;0,400;0,700;0,900;1,100;1,300;1,400;1,700;1,900&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<section>
	<WeatherForecast />
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		align-items: center;
		margin-left: 40px;
		margin-bottom: 40px;
	}
</style>
