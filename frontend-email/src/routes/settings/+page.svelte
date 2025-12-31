<script lang="ts">
	import { onMount } from 'svelte';
	import { Settings, Sparkles, Moon, Sun, Wifi, WifiOff, ArrowLeft, Mail, Link, Link2Off, Check, Loader2 } from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import * as Card from '$lib/components/ui/card';
	import { emailStore } from '$lib/stores/email.svelte';
	import { realtimeStore } from '$lib/stores/realtime.svelte';
	import { emailAPI, type AiQuota } from '$lib/api/client';
	import { goto } from '$app/navigation';

	let darkMode = $state(false);
	let aiQuota = $state<AiQuota | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let timeUntilReset = $state('');

	onMount(async () => {
		darkMode = localStorage.getItem('darkMode') === 'true';
		await loadAiQuota();
		startResetCountdown();
	});

	async function loadAiQuota() {
		loading = true;
		error = null;

		try {
			aiQuota = await emailAPI.getAiQuota(emailStore.accountId);
		} catch (err: any) {
			error = err.message || 'Failed to load AI quota';
			console.error('AI quota error:', err);
		} finally {
			loading = false;
		}
	}

	function startResetCountdown() {
		function updateCountdown() {
			const now = new Date();
			const midnight = new Date(now);
			midnight.setUTCHours(24, 0, 0, 0); // Next midnight UTC

			const diff = midnight.getTime() - now.getTime();
			const hours = Math.floor(diff / (1000 * 60 * 60));
			const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
			const seconds = Math.floor((diff % (1000 * 60)) / 1000);

			timeUntilReset = `${hours}h ${minutes}m ${seconds}s`;
		}

		updateCountdown();
		const interval = setInterval(updateCountdown, 1000);

		return () => clearInterval(interval);
	}

	function toggleDarkMode() {
		darkMode = !darkMode;
		if (typeof window !== 'undefined' && (window as any).toggleDarkMode) {
			(window as any).toggleDarkMode();
		}
	}

	function getUsagePercentage(used: number, limit: number): number {
		return Math.min((used / limit) * 100, 100);
	}

	function getProgressColor(used: number, limit: number): string {
		const percentage = getUsagePercentage(used, limit);
		if (percentage >= 90) return 'bg-red-500';
		if (percentage >= 70) return 'bg-yellow-500';
		return 'bg-green-500';
	}

	function getTextColor(used: number, limit: number): string {
		const percentage = getUsagePercentage(used, limit);
		if (percentage >= 90) return 'text-red-600 dark:text-red-400';
		if (percentage >= 70) return 'text-yellow-600 dark:text-yellow-400';
		return 'text-green-600 dark:text-green-400';
	}

	// OAuth removed - using session-based authentication instead
</script>

<svelte:head>
	<title>Settings - Arack Mail</title>
</svelte:head>

<div class="h-screen flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
	<!-- Header -->
	<header class="flex-shrink-0 h-16" style="background-color: #F8FAFD;">
		<div class="h-full px-6 flex items-center gap-4">
			<!-- Back button -->
			<Button variant="ghost" size="icon" onclick={() => goto('/inbox')}>
				<ArrowLeft class="h-5 w-5" />
			</Button>

			<!-- Logo -->
			<div class="flex items-center">
				<img src="/arackmail.svg" alt="Arack Mail" class="h-6" />
			</div>

			<!-- Title -->
			<div class="flex items-center gap-2 ml-4">
				<Settings class="h-5 w-5 text-gray-700 dark:text-gray-300" />
				<h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Settings</h1>
			</div>

			<!-- Right Actions -->
			<div class="flex items-center gap-2 ml-auto">
				<!-- Connection status indicator -->
				<div
					class="flex items-center gap-1 px-2 py-1 rounded-full text-xs {realtimeStore.connected
						? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
						: realtimeStore.connecting
							? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300'
							: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'}"
				>
					{#if realtimeStore.connected}
						<Wifi class="h-3 w-3" />
						<span>Live</span>
					{:else if realtimeStore.connecting}
						<Wifi class="h-3 w-3 animate-pulse" />
						<span>Connecting...</span>
					{:else}
						<WifiOff class="h-3 w-3" />
						<span>Offline</span>
					{/if}
				</div>
				<Button variant="ghost" size="icon" onclick={toggleDarkMode}>
					{#if darkMode}
						<Sun class="h-5 w-5" />
					{:else}
						<Moon class="h-5 w-5" />
					{/if}
				</Button>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<div class="flex-1 overflow-y-auto">
		<div class="max-w-4xl mx-auto p-8">
			<!-- AI Features Section -->
			<div class="mb-8">
				<div class="flex items-center gap-2 mb-4">
					<Sparkles class="h-6 w-6 text-blue-600 dark:text-blue-400" />
					<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100">
						AI Features Usage
					</h2>
				</div>

				{#if loading}
					<div class="flex items-center justify-center py-12">
						<svg
							class="animate-spin h-8 w-8 text-blue-600"
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
						>
							<circle
								class="opacity-25"
								cx="12"
								cy="12"
								r="10"
								stroke="currentColor"
								stroke-width="4"
							></circle>
							<path
								class="opacity-75"
								fill="currentColor"
								d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
							></path>
						</svg>
					</div>
				{:else if error}
					<Card.Root class="border-red-200 dark:border-red-800">
						<Card.Content class="py-8">
							<p class="text-red-600 dark:text-red-400 text-center">{error}</p>
							<div class="flex justify-center mt-4">
								<Button onclick={loadAiQuota}>Try Again</Button>
							</div>
						</Card.Content>
					</Card.Root>
				{:else if aiQuota}
					<div class="space-y-6">
						<!-- Smart Compose -->
						<Card.Root class="border-gray-200 dark:border-gray-700">
							<Card.Header>
								<Card.Title class="text-lg">Smart Compose</Card.Title>
								<Card.Description>
									AI-powered email completion suggestions while you type
								</Card.Description>
							</Card.Header>
							<Card.Content>
								<div class="space-y-3">
									<div class="flex items-center justify-between">
										<span class="text-sm text-gray-700 dark:text-gray-300">Daily Usage</span>
										<span
											class="text-sm font-semibold {getTextColor(
												aiQuota.smart_compose.used,
												aiQuota.smart_compose.limit
											)}"
										>
											{aiQuota.smart_compose.used} / {aiQuota.smart_compose.limit} calls
										</span>
									</div>
									<!-- Progress bar -->
									<div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
										<div
											class="h-2 rounded-full transition-all {getProgressColor(
												aiQuota.smart_compose.used,
												aiQuota.smart_compose.limit
											)}"
											style="width: {getUsagePercentage(
												aiQuota.smart_compose.used,
												aiQuota.smart_compose.limit
											)}%"
										></div>
									</div>
								</div>
							</Card.Content>
						</Card.Root>

						<!-- Summarization -->
						<Card.Root class="border-gray-200 dark:border-gray-700">
							<Card.Header>
								<Card.Title class="text-lg">Email Summarization</Card.Title>
								<Card.Description>
									Generate concise summaries of email threads with key points
								</Card.Description>
							</Card.Header>
							<Card.Content>
								<div class="space-y-3">
									<div class="flex items-center justify-between">
										<span class="text-sm text-gray-700 dark:text-gray-300">Daily Usage</span>
										<span
											class="text-sm font-semibold {getTextColor(
												aiQuota.summarization.used,
												aiQuota.summarization.limit
											)}"
										>
											{aiQuota.summarization.used} / {aiQuota.summarization.limit} calls
										</span>
									</div>
									<!-- Progress bar -->
									<div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
										<div
											class="h-2 rounded-full transition-all {getProgressColor(
												aiQuota.summarization.used,
												aiQuota.summarization.limit
											)}"
											style="width: {getUsagePercentage(
												aiQuota.summarization.used,
												aiQuota.summarization.limit
											)}%"
										></div>
									</div>
								</div>
							</Card.Content>
						</Card.Root>

						<!-- Priority Ranking -->
						<Card.Root class="border-gray-200 dark:border-gray-700">
							<Card.Header>
								<Card.Title class="text-lg">Priority Inbox</Card.Title>
								<Card.Description>
									AI ranks your emails by importance and urgency
								</Card.Description>
							</Card.Header>
							<Card.Content>
								<div class="space-y-3">
									<div class="flex items-center justify-between">
										<span class="text-sm text-gray-700 dark:text-gray-300">Daily Usage</span>
										<span
											class="text-sm font-semibold {getTextColor(
												aiQuota.priority_ranking.used,
												aiQuota.priority_ranking.limit
											)}"
										>
											{aiQuota.priority_ranking.used} / {aiQuota.priority_ranking.limit} calls
										</span>
									</div>
									<!-- Progress bar -->
									<div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
										<div
											class="h-2 rounded-full transition-all {getProgressColor(
												aiQuota.priority_ranking.used,
												aiQuota.priority_ranking.limit
											)}"
											style="width: {getUsagePercentage(
												aiQuota.priority_ranking.used,
												aiQuota.priority_ranking.limit
											)}%"
										></div>
									</div>
								</div>
							</Card.Content>
						</Card.Root>

						<!-- Reset Info -->
						<div
							class="bg-blue-50 dark:bg-blue-950/30 border border-blue-200 dark:border-blue-800 rounded-lg p-4"
						>
							<div class="flex items-start gap-3">
								<Sparkles class="h-5 w-5 text-blue-600 dark:text-blue-400 mt-0.5" />
								<div class="flex-1">
									<p class="text-sm font-medium text-blue-900 dark:text-blue-100 mb-1">
										Quota Reset
									</p>
									<p class="text-sm text-blue-700 dark:text-blue-300">
										All AI feature quotas reset daily at midnight UTC. Time until reset:
										<span class="font-semibold">{timeUntilReset}</span>
									</p>
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>

			<!-- OAuth Connection Section removed - using session-based authentication -->

			<!-- General Settings Section -->
			<div class="mb-8">
				<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">
					General Settings
				</h2>

				<Card.Root class="border-gray-200 dark:border-gray-700">
					<Card.Header>
						<Card.Title class="text-lg">Appearance</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="flex items-center justify-between">
							<div>
								<p class="font-medium text-gray-900 dark:text-gray-100">Dark Mode</p>
								<p class="text-sm text-gray-600 dark:text-gray-400">
									Toggle between light and dark themes
								</p>
							</div>
							<Button variant="outline" onclick={toggleDarkMode}>
								{#if darkMode}
									<Sun class="h-4 w-4 mr-2" />
									Light
								{:else}
									<Moon class="h-4 w-4 mr-2" />
									Dark
								{/if}
							</Button>
						</div>
					</Card.Content>
				</Card.Root>
			</div>

			<!-- Account Info Section -->
			<div class="mb-8">
				<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-4">Account</h2>

				<Card.Root class="border-gray-200 dark:border-gray-700">
					<Card.Header>
						<Card.Title class="text-lg">Email Account</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="space-y-4">
							<div>
								<p class="text-sm text-gray-600 dark:text-gray-400">Email Address</p>
								<p class="font-medium text-gray-900 dark:text-gray-100">
									{emailStore.currentAccount?.email_address || 'Not loaded'}
								</p>
							</div>
							<div>
								<p class="text-sm text-gray-600 dark:text-gray-400">Account ID</p>
								<p class="font-mono text-sm text-gray-900 dark:text-gray-100">
									{emailStore.accountId}
								</p>
							</div>
						</div>
					</Card.Content>
				</Card.Root>
			</div>
		</div>
	</div>
</div>
