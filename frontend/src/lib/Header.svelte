<script lang="ts">
	import { page } from '$app/stores';
</script>

<header>
	<h1>Home Control <sup>v1</sup></h1>
	<!--<li class:active={$page.path === '/'}><a sveltekit:prefetch href="/">Home</a></li>-->
</header>

<style lang="scss">
	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		height: 96px;
		padding: 0 24px;
		background-color: var(--menu-background-color);

		h1 {
			font-size: 170%;
			font-weight: 800;
			letter-spacing: -2px;

			sup {
				font-style: italic;
				color: var(--gray);
			}
		}
	}
</style>
