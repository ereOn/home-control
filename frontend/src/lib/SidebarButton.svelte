<script lang="ts">
	export let icon: string;

	import Icon from '@iconify/svelte';
</script>

<button><Icon {icon} style="font-size: 48px" /></button>

<style lang="scss">
	button {
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: 8px;
		background-color: var(--button-background-color-off);
		background: radial-gradient(
			ellipse at center,
			white 20%,
			var(--button-background-center-color-off) 100%
		);
		color: var(--button-text-color);
		height: 80px;
		width: 80px;
		font-weight: 800;
		font-size: 120%;
		user-select: none;

		&:active {
			filter: brightness(0.8);
		}
	}
</style>
