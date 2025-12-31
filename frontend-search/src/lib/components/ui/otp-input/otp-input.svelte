<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		value?: string;
		length?: number;
		disabled?: boolean;
		class?: string;
		oninput?: (value: string) => void;
	}

	let { value = $bindable(''), length = 6, disabled = false, class: className, oninput }: Props = $props();

	let digits = $state<string[]>(Array(length).fill(''));
	let inputRefs: HTMLInputElement[] = [];

	// Sync digits with value
	$effect(() => {
		const valueArray = value.split('').slice(0, length);
		digits = [...valueArray, ...Array(length - valueArray.length).fill('')];
	});

	function handleInput(index: number, event: Event) {
		const input = event.target as HTMLInputElement;
		const newValue = input.value;

		// Only allow single digit
		if (newValue.length > 1) {
			input.value = newValue.slice(-1);
		}

		// Update digits array
		digits[index] = input.value;
		value = digits.join('');

		// Call oninput callback
		if (oninput) {
			oninput(value);
		}

		// Auto-advance to next input
		if (input.value && index < length - 1) {
			inputRefs[index + 1]?.focus();
		}
	}

	function handleKeydown(index: number, event: KeyboardEvent) {
		// Handle backspace to go to previous input
		if (event.key === 'Backspace' && !digits[index] && index > 0) {
			inputRefs[index - 1]?.focus();
		}

		// Handle arrow keys
		if (event.key === 'ArrowLeft' && index > 0) {
			event.preventDefault();
			inputRefs[index - 1]?.focus();
		}

		if (event.key === 'ArrowRight' && index < length - 1) {
			event.preventDefault();
			inputRefs[index + 1]?.focus();
		}
	}

	function handlePaste(event: ClipboardEvent) {
		event.preventDefault();
		const pastedData = event.clipboardData?.getData('text') || '';
		const pastedDigits = pastedData.replace(/\D/g, '').slice(0, length);

		if (pastedDigits) {
			digits = pastedDigits.split('');
			// Fill remaining with empty strings
			while (digits.length < length) {
				digits.push('');
			}
			value = digits.join('');

			// Call oninput callback
			if (oninput) {
				oninput(value);
			}

			// Focus the next empty input or last input
			const nextEmptyIndex = digits.findIndex((d) => !d);
			if (nextEmptyIndex !== -1) {
				inputRefs[nextEmptyIndex]?.focus();
			} else {
				inputRefs[length - 1]?.focus();
			}
		}
	}
</script>

<div class={cn('flex gap-2 justify-center', className)} onpaste={handlePaste}>
	{#each Array(length) as _, index}
		<input
			bind:this={inputRefs[index]}
			type="text"
			inputmode="numeric"
			pattern="[0-9]"
			maxlength="1"
			value={digits[index]}
			{disabled}
			oninput={(e) => handleInput(index, e)}
			onkeydown={(e) => handleKeydown(index, e)}
			class={cn(
				'w-11 h-12 text-center text-xl font-semibold rounded-md border transition-colors',
				'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gray-950',
				'disabled:opacity-50 disabled:cursor-not-allowed',
				'border-gray-300 bg-white text-gray-900',
				'hover:border-gray-400'
			)}
			autocomplete="off"
		/>
	{/each}
</div>
