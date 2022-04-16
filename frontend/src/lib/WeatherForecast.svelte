<script lang="ts">
	let temperature = 25;
	let city = "L'Assomption";
	let weather = 'Cloudy';
</script>

<div>
	<h1>{temperature}°</h1>
	<span class="details">
		<h2>{city}</h2>
		<p>{weather}</p>
	</span>
</div>

<style lang="scss">
	div {
		display: flex;
		flex-direction: row;

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
