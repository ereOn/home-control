<script>
	import { api } from './api';
</script>

<div>
	{#if $api.status === 'disconnected'}
		<div class="disconnected blink" />
	{/if}
	<slot />
</div>

<style lang="scss">
	div {
		display: flex;
		flex: 1;
		flex-direction: column;
		margin: 0;
		padding: 0;
		position: relative;

		div.disconnected {
			position: absolute;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			z-index: 1;
			background: url('/icons/disconnected.jpg') center center no-repeat rgba(0, 0, 0, 0.5);
			background-size: 192px;
			pointer-events: none;
		}
	}
</style>
