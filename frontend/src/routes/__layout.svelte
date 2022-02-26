<script lang="ts">
	import Header from '$lib/Header.svelte';
	import Sidebar from '$lib/Sidebar.svelte';
	import '../app.css';
</script>

<main>
	<div id="content">
		<Header />
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
	}

	main > div#content {
		flex: 1;
		padding: 0;
		margin: 0;
		box-sizing: border-box;
	}

	main > div#content > slot {
		flex: 1;
	}
</style>
