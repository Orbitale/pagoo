<script lang="ts">
	import 'bootstrap/dist/css/bootstrap.min.css';
	import bootstrapJs from 'bootstrap/dist/js/bootstrap.bundle.js?url';

	import favicon from '$lib/assets/favicon.svg';
	import Header from "$lib/UI/Header.svelte";
	import Sidebar from "$lib/UI/Sidebar.svelte";

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<script src="{bootstrapJs}"></script>
</svelte:head>

<Header />

<div class="container-fluid">
	<div class="row">
		<div class="sidebar border border-right col-md-3 col-lg-2 p-0 bg-body-tertiary">
			<Sidebar />
		</div>
		<main class="col-md-9 ms-sm-auto col-lg-10 px-md-4 mt-md-4">
			{@render children()}
		</main>
	</div>
</div>
