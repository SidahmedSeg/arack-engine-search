# Custom UI Components Guide

## Overview

This project uses a **manual component system** instead of pre-built UI libraries (shadcn-svelte, Skeleton UI, Flowbite).

**Why Manual Components?**
- Modern Svelte UI libraries require Tailwind v4, which needs Vite 5-6
- We're running Vite 7, making those libraries incompatible
- Manual components give exact control over design
- No dependency conflicts
- Tailored to "compact, modern, minimal" design requirements

## Design Philosophy

### Visual Style: Google-like Minimalism
- **Compact spacing**: Smaller heights (h-9 instead of h-10/h-11)
- **Minimal borders**: Simple gray-300 borders, no heavy shadows
- **Clean focus states**: ring-1 instead of ring-2
- **System fonts**: -apple-system, BlinkMacSystemFont, Segoe UI, Roboto
- **Consistent colors**: Use `text-primary` utility instead of hardcoded blues

### Technical Approach
- **Svelte 5 runes**: Use `$state`, `$bindable`, `$props`, `$effect`
- **Svelte 5 snippets**: For children rendering
- **Tailwind CSS v3**: Current version (no v4)
- **cn() utility**: Class name merging with clsx + tailwind-merge
- **TypeScript**: Strongly typed component props

## Project Structure

```
src/lib/
├── utils.ts                    # cn() utility function
├── components/ui/
│   ├── button/
│   │   └── button.svelte      # Button component
│   ├── input/
│   │   └── input.svelte       # Input component with label/error
│   ├── label/
│   │   └── label.svelte       # Form label
│   ├── card/
│   │   ├── index.ts           # Namespace exports
│   │   ├── card.svelte        # Card root
│   │   ├── card-header.svelte
│   │   ├── card-title.svelte
│   │   ├── card-description.svelte
│   │   └── card-content.svelte
│   └── otp-input/
│       └── otp-input.svelte   # OTP code input (6 digits)
```

## Core Utilities

### cn() - Class Name Utility

**Location**: `src/lib/utils.ts`

```typescript
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}
```

**Usage**: Merge Tailwind classes, handle conditional classes, resolve conflicts

```svelte
<div class={cn('base-class', isActive && 'active-class', className)} />
```

## Component Patterns

### 1. Simple Component (Button, Label)

**Key Features**:
- Single `.svelte` file
- Props interface with TypeScript
- Svelte 5 runes (`$props`)
- Variant system for different styles
- cn() for class merging

**Example**: `src/lib/components/ui/button/button.svelte`

```svelte
<script lang="ts">
	import { type Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	interface Props {
		variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
		size?: 'default' | 'sm' | 'lg' | 'icon';
		class?: string;
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		onclick?: (event: MouseEvent) => void;
		children?: Snippet;
	}

	let {
		variant = 'default',
		size = 'default',
		class: className,
		type = 'button',
		disabled = false,
		onclick,
		children,
		...restProps
	}: Props = $props();

	// Variant mapping
	const variants = {
		default: 'bg-primary text-white hover:bg-primary/90',
		destructive: 'bg-red-600 text-white hover:bg-red-700',
		outline: 'border border-gray-300 bg-white hover:bg-gray-50 text-gray-900',
		secondary: 'bg-gray-100 text-gray-900 hover:bg-gray-200',
		ghost: 'hover:bg-gray-100 hover:text-gray-900',
		link: 'text-primary underline-offset-4 hover:underline'
	};

	const sizes = {
		default: 'h-9 px-4 py-2',
		sm: 'h-8 px-3 text-sm',
		lg: 'h-10 px-6',
		icon: 'h-9 w-9'
	};
</script>

<button
	{type}
	{disabled}
	{onclick}
	class={cn(
		'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors',
		'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gray-950',
		'disabled:pointer-events-none disabled:opacity-50',
		variants[variant],
		sizes[size],
		className
	)}
	{...restProps}
>
	{#if children}{@render children()}{/if}
</button>
```

### 2. Form Component with $bindable (Input)

**Key Features**:
- Two-way binding with `$bindable()`
- Optional label and error display
- Accessible form structure

**Example**: `src/lib/components/ui/input/input.svelte`

```svelte
<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		type?: 'text' | 'email' | 'password' | 'number' | 'tel' | 'url' | 'search';
		value?: string;
		name?: string;
		id?: string;
		placeholder?: string;
		required?: boolean;
		disabled?: boolean;
		label?: string;
		error?: string;
		class?: string;
		oninput?: (e: Event) => void;
	}

	let {
		type = 'text',
		value = $bindable(''),
		name,
		id,
		placeholder,
		required = false,
		disabled = false,
		label,
		error,
		class: className,
		oninput,
		...restProps
	}: Props = $props();
</script>

{#if label}
	<label for={id} class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
		{label}
		{#if required}<span class="text-red-500 ml-0.5">*</span>{/if}
	</label>
{/if}

<input
	{type}
	{name}
	{id}
	{placeholder}
	{required}
	{disabled}
	{oninput}
	bind:value
	class={cn(
		'flex h-9 w-full rounded-md border bg-white px-3 py-1 text-sm transition-colors',
		'file:border-0 file:bg-transparent file:text-sm file:font-medium',
		'placeholder:text-gray-500',
		'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gray-950',
		'disabled:cursor-not-allowed disabled:opacity-50',
		error ? 'border-red-500 focus-visible:ring-red-500' : 'border-gray-300',
		className
	)}
	{...restProps}
/>

{#if error}
	<p class="mt-1.5 text-xs text-red-600 dark:text-red-400">{error}</p>
{/if}
```

### 3. Compound Component (Card)

**Key Features**:
- Multiple sub-components
- Namespace export pattern
- Composition-based usage

**Structure**:
```
card/
├── index.ts               # Exports
├── card.svelte           # <Card.Root>
├── card-header.svelte    # <Card.Header>
├── card-title.svelte     # <Card.Title>
├── card-description.svelte # <Card.Description>
└── card-content.svelte   # <Card.Content>
```

**Example**: `src/lib/components/ui/card/index.ts`

```typescript
import Root from './card.svelte';
import Content from './card-content.svelte';
import Description from './card-description.svelte';
import Header from './card-header.svelte';
import Title from './card-title.svelte';

export {
	Root,
	Content,
	Description,
	Header,
	Title,
	// Also export with full names for convenience
	Root as Card,
	Content as CardContent,
	Description as CardDescription,
	Header as CardHeader,
	Title as CardTitle
};
```

**Usage**:

```svelte
<script>
	import * as Card from '$lib/components/ui/card';
</script>

<Card.Root class="w-full max-w-md">
	<Card.Header>
		<Card.Title>Card Title</Card.Title>
		<Card.Description>Card description text</Card.Description>
	</Card.Header>
	<Card.Content>
		<p>Card content goes here</p>
	</Card.Content>
</Card.Root>
```

### 4. Interactive Component (OTP Input)

**Key Features**:
- State management with `$state`
- Effects with `$effect`
- Event handling (input, keydown, paste)
- Auto-advance between inputs

**Example**: `src/lib/components/ui/otp-input/otp-input.svelte`

```svelte
<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		value?: string;
		length?: number;
		disabled?: boolean;
		class?: string;
		oninput?: (value: string) => void;
	}

	let {
		value = $bindable(''),
		length = 6,
		disabled = false,
		class: className,
		oninput
	}: Props = $props();

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

		if (newValue.length > 1) {
			input.value = newValue.slice(-1);
		}

		digits[index] = input.value;
		value = digits.join('');

		if (oninput) {
			oninput(value);
		}

		// Auto-advance
		if (input.value && index < length - 1) {
			inputRefs[index + 1]?.focus();
		}
	}

	function handleKeydown(index: number, event: KeyboardEvent) {
		if (event.key === 'Backspace' && !digits[index] && index > 0) {
			inputRefs[index - 1]?.focus();
		}

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
			while (digits.length < length) {
				digits.push('');
			}
			value = digits.join('');

			if (oninput) {
				oninput(value);
			}

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
```

## Design System

### Colors

Use Tailwind's color palette with these conventions:

```typescript
// Primary color (defined in tailwind.config.js)
primary: '#4285F4'  // Google blue

// Usage in components
'text-primary'           // Primary text
'bg-primary'            // Primary background
'border-primary'        // Primary border
'hover:bg-primary/90'   // Hover state with opacity
```

### Spacing (Compact)

```css
/* Custom spacing variables in app.css */
--spacing-xs: 0.25rem;  /* 4px */
--spacing-sm: 0.5rem;   /* 8px */
--spacing-md: 1rem;     /* 16px */
--spacing-lg: 1.5rem;   /* 24px */
--spacing-xl: 2rem;     /* 32px */
```

### Component Sizes

```typescript
// Input/Button heights
h-8   // Small: 32px
h-9   // Default: 36px (our standard)
h-10  // Large: 40px

// Padding
px-3 py-1  // Input: 12px horizontal, 4px vertical
px-4 py-2  // Button: 16px horizontal, 8px vertical
```

### Typography

```css
/* Font stack (in app.css) */
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;

/* Font sizes */
text-xs    /* 12px */
text-sm    /* 14px */
text-base  /* 16px */
text-lg    /* 18px */
text-xl    /* 20px */
text-2xl   /* 24px */
```

### Focus States

```typescript
// Standard focus ring (minimal)
'focus-visible:outline-none'
'focus-visible:ring-1'           // Thin ring (not ring-2)
'focus-visible:ring-gray-950'

// Error focus
'focus-visible:ring-red-500'
```

## Creating New Components

### Step-by-Step Guide

1. **Create component directory**:
   ```bash
   mkdir -p src/lib/components/ui/my-component
   ```

2. **Create component file**:
   ```bash
   touch src/lib/components/ui/my-component/my-component.svelte
   ```

3. **Component template**:
   ```svelte
   <script lang="ts">
   	import { cn } from '$lib/utils';
   	import { type Snippet } from 'svelte';

   	interface Props {
   		class?: string;
   		children?: Snippet;
   		// Add your props here
   	}

   	let { class: className, children, ...restProps }: Props = $props();
   </script>

   <div
   	class={cn(
   		// Base classes
   		'base-class',
   		// Add variants if needed
   		className
   	)}
   	{...restProps}
   >
   	{#if children}{@render children()}{/if}
   </div>
   ```

4. **For compound components, create index.ts**:
   ```typescript
   import Root from './my-component.svelte';
   import SubComponent from './my-component-sub.svelte';

   export {
   	Root,
   	SubComponent,
   	Root as MyComponent,
   	SubComponent as MyComponentSub
   };
   ```

## Component Checklist

When creating a new component, ensure:

- [ ] Uses Svelte 5 syntax (`$props`, `$state`, `$bindable`)
- [ ] Has TypeScript Props interface
- [ ] Uses `cn()` for class merging
- [ ] Accepts `class` prop for custom styling
- [ ] Spreads `...restProps` for HTML attributes
- [ ] Follows compact sizing (h-9 for inputs/buttons)
- [ ] Uses minimal borders (border-gray-300)
- [ ] Has proper focus states (ring-1)
- [ ] Accessible (labels, ARIA attributes if needed)
- [ ] Dark mode support (dark: classes)

## Common Patterns

### Conditional Classes

```svelte
<div class={cn(
	'base',
	isActive && 'active-class',
	error && 'error-class',
	className
)} />
```

### Variant Systems

```typescript
const variants = {
	default: 'default-classes',
	primary: 'primary-classes',
	danger: 'danger-classes'
};

// In template
class={cn(variants[variant], className)}
```

### Two-Way Binding

```typescript
let { value = $bindable(''), ...restProps }: Props = $props();

// In template
<input bind:value />
```

### Children with Snippets

```typescript
import { type Snippet } from 'svelte';

interface Props {
	children?: Snippet;
}

let { children }: Props = $props();

// In template
{#if children}{@render children()}{/if}
```

## Examples from Current Codebase

### Button Variants

```svelte
<!-- Default primary button -->
<Button variant="default">Save</Button>

<!-- Outline button -->
<Button variant="outline">Cancel</Button>

<!-- Destructive button -->
<Button variant="destructive">Delete</Button>

<!-- Link-style button -->
<Button variant="link">Learn more</Button>
```

### Input with Label and Error

```svelte
<Input
	type="email"
	label="Email Address"
	placeholder="you@example.com"
	required
	bind:value={email}
	error={emailError}
/>
```

### Card Composition

```svelte
<Card.Root class="max-w-md">
	<Card.Header>
		<Card.Title>Welcome</Card.Title>
		<Card.Description>Sign in to continue</Card.Description>
	</Card.Header>
	<Card.Content>
		<!-- Form or content -->
	</Card.Content>
</Card.Root>
```

### OTP Input

```svelte
<OTPInput
	length={6}
	bind:value={code}
	oninput={(value) => {
		if (value.length === 6) {
			handleSubmit();
		}
	}}
/>
```

## Troubleshooting

### Component Not Rendering

- Check imports are correct
- Verify component file exists at expected path
- Check for TypeScript errors in Props interface

### Styles Not Applying

- Ensure Tailwind classes are spelled correctly
- Check `cn()` utility is imported
- Verify className is being spread: `class={cn(..., className)}`
- Clear Tailwind cache: `npx tailwindcss -i src/app.css -o dist/output.css --watch`

### Dark Mode Not Working

- Add `dark:` prefix to classes: `dark:bg-gray-800`
- Check `dark` class is on `<html>` or parent element
- Verify Tailwind config has `darkMode: 'class'`

## Best Practices

1. **Keep components focused** - One responsibility per component
2. **Composition over configuration** - Prefer compound components over complex props
3. **Consistent naming** - Use kebab-case for files, PascalCase for components
4. **Accessibility first** - Always add proper labels, ARIA attributes
5. **Progressive enhancement** - Components should work without JavaScript where possible
6. **Type safety** - Always define Props interfaces
7. **Document variants** - Add comments explaining variant options
8. **Test in both themes** - Check light and dark modes
9. **Responsive design** - Use Tailwind's responsive prefixes (sm:, md:, lg:)
10. **Performance** - Avoid unnecessary reactivity, use `$effect` sparingly

## Migration Guide

### From Old Custom Components

If migrating from old custom components to new structure:

1. **Update imports**:
   ```svelte
   <!-- Old -->
   import Card from '$lib/components/ui/Card.svelte';

   <!-- New -->
   import * as Card from '$lib/components/ui/card';
   ```

2. **Update markup**:
   ```svelte
   <!-- Old -->
   <Card class="w-full">
   	<p>Content</p>
   </Card>

   <!-- New -->
   <Card.Root class="w-full">
   	<Card.Content>
   		<p>Content</p>
   	</Card.Content>
   </Card.Root>
   ```

3. **Update variant names**:
   ```svelte
   <!-- Old -->
   <Button variant="primary">Save</Button>

   <!-- New -->
   <Button variant="default">Save</Button>
   ```

## Resources

- **Svelte 5 Docs**: https://svelte.dev/docs/svelte/overview
- **Tailwind CSS**: https://tailwindcss.com/docs
- **TypeScript**: https://www.typescriptlang.org/docs/
- **clsx**: https://github.com/lukeed/clsx
- **tailwind-merge**: https://github.com/dcastil/tailwind-merge

## Support

For questions or issues with the component system:
1. Check this guide first
2. Review existing components for examples
3. Check Svelte 5 documentation for syntax
4. Consult Tailwind docs for styling
