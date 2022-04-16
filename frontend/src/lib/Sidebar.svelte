<script>
	import SidebarButton from './SidebarButton.svelte';
	import SidebarLightButton from './SidebarLightButton.svelte';
</script>

<div id="sidebar">
	<h2>Scenes</h2>
	<SidebarLightButton icon="noto:pool-8-ball" name="pool" />
	<SidebarLightButton icon="noto:books" name="reading" />
	<div class="separator" />
	<SidebarButton icon="emojione:film-projector" />
	<div class="filler" />
	<SidebarButton icon="emojione:bed" />
</div>

<style lang="scss">
	div#sidebar {
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: 424px;
		background-color: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(4px);
		justify-content: end;
		align-items: left;

		* {
			margin: 8px;
		}

		h2 {
			font-weight: 300;
			padding-top: 8px;
			padding-bottom: 16px;
			border-bottom: 1px solid rgba(255, 255, 255, 0.3);
		}

		div.separator {
			height: 20px;
		}
		div.filler {
			flex: 1;
		}
	}
</style>
