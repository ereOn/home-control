<script lang="ts">
	import Sidebar from '$lib/Sidebar.svelte';
	import Connectivity from '$lib/Connectivity.svelte';
	import '../app.css';
	import { api } from '../lib/api';

	$: weatherCurrent = $api.status.status === 'connected' ? $api.status.weatherCurrent.state : '';
	$: weatherForecast = $api.status.status === 'connected' ? $api.status.weatherForecast.state : '';

	let rootElement;

	$: if (rootElement) {
		rootElement.style.setProperty(
			'--weather-current-image-url',
			'url("/backgrounds/weather/' + weatherCurrent + '.jpg")'
		);
		rootElement.style.setProperty(
			'--weather-forecast-image-url',
			'url("/backgrounds/weather/' + weatherForecast + '.jpg")'
		);
	}
</script>

<Connectivity>
	<main bind:this={rootElement}>
		<div class="weather weather-forecast" />
		<div class="weather weather-current" />

		<div id="content">
			<slot />
		</div>

		<Sidebar />
	</main>
</Connectivity>

<style lang="scss">
	:root {
		--weather-current-image-url: '';
		--weather-forecast-image-url: '';
	}

	main {
		position: relative;
		flex: 1;
		display: flex;
		flex-direction: row;
		padding: 0;
		width: 100%;
		height: 100%;
		margin: 0 auto;
		box-sizing: border-box;
		justify-content: space-between;
		align-items: stretch;
		color: white;

		& > .weather {
			position: absolute;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;

			background-position: center 25%;
			background-repeat: no-repeat;
			background-size: cover;
			z-index: 0;

			&.weather-forecast {
				background-image: var(--weather-forecast-image-url);
			}

			&.weather-current {
				background-image: var(--weather-current-image-url);
				mask-image: linear-gradient(
					106deg,
					rgba(255, 255, 255, 1) 0%,
					rgba(255, 255, 255, 1) 60%,
					rgba(0, 0, 0, 0) 70%,
					rgba(0, 0, 0, 0) 100%
				);
				-webkit-mask-image: linear-gradient(
					106deg,
					rgba(255, 255, 255, 1) 0%,
					rgba(255, 255, 255, 1) 60%,
					rgba(0, 0, 0, 0) 70%,
					rgba(0, 0, 0, 0) 100%
				);
			}
		}

		> div#content {
			flex: 1;
			display: flex;
			flex-direction: column;
			align-items: flex-start;
			justify-items: flex-end;
			justify-content: flex-end;
			margin: 16px;
			z-index: 1;
		}
	}
</style>
