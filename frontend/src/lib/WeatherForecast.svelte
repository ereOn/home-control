<script lang="ts">
	import { api } from './api';

	$: weatherCurrentLabel =
		$api.status.status === 'connected'
			? {
					'clear-night': 'Nuit dégagée',
					cloudy: 'Nuageux',
					fog: 'Brumeux',
					hail: 'Grêle',
					lightning: 'Orage',
					'lighting-rainy': 'Pluie orageuse',
					partlycloudy: 'Partiellement nuageux',
					pouring: 'Pluie forte',
					rainy: 'Pluvieux',
					snowy: 'Neigeux',
					'snowy-rainy': 'Pluie verglaçante',
					sunny: 'Ensoleillé',
					windy: 'Venteux',
					'windy-variant': 'Vents variables',
					exceptional: 'Inhabituel'
			  }[$api.status.weatherCurrent.state]
			: '';
</script>

<div>
	{#if $api.status.status === 'connected'}
		<h1>{$api.status.weatherCurrent.temperature}°</h1>
		<span class="details">
			<h2>{$api.status.location}</h2>
			<p>{weatherCurrentLabel}</p>
		</span>
	{/if}
</div>

<style lang="scss">
	div {
		display: flex;
		flex-direction: row;
		text-shadow: 6px 6px 12px rgba(0, 0, 0, 0.4);

		h1 {
			font-size: 550%;
			font-weight: 900;
			margin: 0;
			margin-right: 24px;
		}

		span {
			display: flex;
			flex-direction: column;
			justify-content: center;

			h2 {
				font-size: 250%;
				font-weight: 500;
				margin: 0;
			}

			p {
				font-size: 150%;
				font-weight: 300;
				margin: 0;
			}
		}
	}
</style>
