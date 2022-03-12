<script>
	import SidebarButton from './SidebarButton.svelte';
	import SidebarLightButton from './SidebarLightButton.svelte';
</script>

<div id="sidebar">
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
	}
</style>
