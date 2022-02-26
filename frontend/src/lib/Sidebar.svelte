<script>
	import Icon from '@iconify/svelte';
</script>

<div id="sidebar">
	<button><Icon icon="noto:pool-8-ball" style="font-size: 48px" /></button>
	<button><Icon icon="noto:books" style="font-size: 48px" /></button>
	<div class="separator" />
	<button><Icon icon="emojione:film-projector" style="font-size: 48px" /></button>
	<div class="filler" />
	<button><Icon icon="emojione:bed" style="font-size: 48px" /></button>
</div>

<style lang="scss">
	div#sidebar {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px 0;
		width: 96px;
		border-left: 1px solid var(--shadow-color);
		box-shadow: 8px 0px 16px 16px var(--shadow-color);
		background-color: white;
		justify-content: end;
		align-items: center;

		div.separator {
			height: 20px;
		}
		div.filler {
			flex: 1;
		}

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
	}
</style>
