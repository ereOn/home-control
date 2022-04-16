<script lang="ts">
	import Sidebar from '$lib/Sidebar.svelte';
	import '../app.css';

	let weather = 'rainy';
</script>

<main class={weather}>
	<div id="content">
		<slot />
	</div>

	<Sidebar />
</main>

<style>
	main {
		flex: 1;
		display: flex;
		flex-direction: row;
		padding: 0;
		width: 100%;
		margin: 0 auto;
		box-sizing: border-box;
		height: 100%;
		justify-content: space-between;
		align-items: stretch;
		color: white;
		background-position: center 25%;
		background-repeat: no-repeat;
		background-size: cover;
	}

	main > div#content {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		justify-items: flex-end;
		margin: 16px;
	}

	main.rainy {
		background-image: url('/backgrounds/weather/rainy.jpg');
	}

	main.sunny {
		background-image: url('/backgrounds/weather/sunny.jpg');
	}

	main.cloudy {
		background-image: url('/backgrounds/weather/cloudy.jpg');
	}
</style>
