<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Chart,
		CategoryScale,
		LinearScale,
		BarElement,
		LineElement,
		PointElement,
		ArcElement,
		Title,
		Tooltip,
		Legend,
		type ChartConfiguration
	} from 'chart.js';

	// Register Chart.js components
	Chart.register(
		CategoryScale,
		LinearScale,
		BarElement,
		LineElement,
		PointElement,
		ArcElement,
		Title,
		Tooltip,
		Legend
	);

	interface Props {
		config: ChartConfiguration;
		height?: number;
	}

	let { config, height = 300 }: Props = $props();

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;

	onMount(() => {
		if (canvas) {
			chart = new Chart(canvas, config);
		}
	});

	onDestroy(() => {
		if (chart) {
			chart.destroy();
		}
	});

	// Update chart when config changes
	$effect(() => {
		if (chart && config) {
			chart.data = config.data;
			chart.options = config.options;
			chart.update();
		}
	});
</script>

<div style="height: {height}px; position: relative;">
	<canvas bind:this={canvas}></canvas>
</div>
